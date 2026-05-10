use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration as StdDuration,
};

use chrono::{DateTime, Duration, Utc};
use persistence::{CardPrice, PersistenceSystemTrait};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{GathersState, StorageState, mtg_api::mtg_api_models::APICardPrice};

pub const PRICE_SOURCE_SCRYFALL: &str = "scryfall";
const SEARCH_PRICE_TTL_DAYS: i64 = 7;
const BATCH_SIZE: usize = 75;

pub fn api_price_from_cache(price: &CardPrice) -> APICardPrice {
    APICardPrice {
        usd_cents: price.usd_cents,
        usd_foil_cents: price.usd_foil_cents,
        usd_etched_cents: price.usd_etched_cents,
        fetched_at: price.fetched_at.clone(),
    }
}

pub async fn cached_prices_for_scryfall_ids(
    state: &GathersState,
    scryfall_ids: impl IntoIterator<Item = String>,
    priority: i32,
) -> eyre::Result<HashMap<String, CardPrice>> {
    let mut seen = HashSet::new();
    let ids: Vec<String> = scryfall_ids
        .into_iter()
        .filter(|id| !id.is_empty())
        .filter(|id| seen.insert(id.clone()))
        .collect();

    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    let prices = {
        let storage = &state.1.lock().await.storage;
        storage.get_card_prices(PRICE_SOURCE_SCRYFALL, &ids).await?
    };

    let stale_or_missing: Vec<String> = ids
        .iter()
        .filter(|id| {
            prices
                .get(*id)
                .map(price_is_stale)
                .unwrap_or(true)
        })
        .cloned()
        .collect();

    if !stale_or_missing.is_empty() {
        state
            .1
            .lock()
            .await
            .storage
            .enqueue_card_price_refresh(PRICE_SOURCE_SCRYFALL, &stale_or_missing, priority)
            .await?;
    }

    Ok(prices)
}

fn price_is_stale(price: &CardPrice) -> bool {
    DateTime::parse_from_rfc3339(&price.fetched_at)
        .map(|dt| Utc::now() - dt.with_timezone(&Utc) > Duration::days(SEARCH_PRICE_TTL_DAYS))
        .unwrap_or(true)
}

pub fn spawn_scryfall_price_worker(storage: Arc<Mutex<StorageState>>) {
    tokio::spawn(async move {
        let client = reqwest::Client::new();

        loop {
            let ids = match storage
                .lock()
                .await
                .storage
                .take_card_price_refresh_batch(PRICE_SOURCE_SCRYFALL, BATCH_SIZE)
                .await
            {
                Ok(ids) => ids,
                Err(e) => {
                    eprintln!("Failed to read Scryfall price refresh queue: {e}");
                    tokio::time::sleep(StdDuration::from_secs(5)).await;
                    continue;
                }
            };

            if ids.is_empty() {
                tokio::time::sleep(StdDuration::from_millis(500)).await;
                continue;
            }

            match fetch_scryfall_prices(&client, &ids).await {
                Ok(prices) => {
                    let mut storage = storage.lock().await;
                    if let Err(e) = storage.storage.upsert_card_prices(&prices).await {
                        eprintln!("Failed to cache Scryfall prices: {e}");
                        let _ = storage
                            .storage
                            .fail_card_price_refresh(PRICE_SOURCE_SCRYFALL, &ids)
                            .await;
                    } else if let Err(e) = storage
                        .storage
                        .complete_card_price_refresh(PRICE_SOURCE_SCRYFALL, &ids)
                        .await
                    {
                        eprintln!("Failed to complete Scryfall price refresh rows: {e}");
                    }
                }
                Err(PriceFetchError::RateLimited) => {
                    tokio::time::sleep(StdDuration::from_secs(10)).await;
                }
                Err(PriceFetchError::Other(e)) => {
                    eprintln!("Failed to refresh Scryfall prices: {e}");
                    let _ = storage
                        .lock()
                        .await
                        .storage
                        .fail_card_price_refresh(PRICE_SOURCE_SCRYFALL, &ids)
                        .await;
                    tokio::time::sleep(StdDuration::from_secs(2)).await;
                }
            }

            tokio::time::sleep(StdDuration::from_millis(650)).await;
        }
    });
}

#[derive(Debug)]
enum PriceFetchError {
    RateLimited,
    Other(eyre::Report),
}

#[derive(Serialize)]
struct ScryfallCollectionRequest {
    identifiers: Vec<ScryfallIdentifier>,
}

#[derive(Serialize)]
struct ScryfallIdentifier {
    id: String,
}

#[derive(Deserialize)]
struct ScryfallCollectionResponse {
    data: Vec<ScryfallCard>,
}

#[derive(Deserialize)]
struct ScryfallCard {
    id: String,
    prices: ScryfallPrices,
}

#[derive(Deserialize)]
struct ScryfallPrices {
    usd: Option<String>,
    usd_foil: Option<String>,
    usd_etched: Option<String>,
}

async fn fetch_scryfall_prices(
    client: &reqwest::Client,
    ids: &[String],
) -> Result<Vec<CardPrice>, PriceFetchError> {
    let body = ScryfallCollectionRequest {
        identifiers: ids
            .iter()
            .map(|id| ScryfallIdentifier { id: id.clone() })
            .collect(),
    };

    let response = client
        .post("https://api.scryfall.com/cards/collection")
        .header(reqwest::header::USER_AGENT, "gathers/1.0")
        .header(reqwest::header::ACCEPT, "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| PriceFetchError::Other(e.into()))?;

    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        return Err(PriceFetchError::RateLimited);
    }
    if !response.status().is_success() {
        return Err(PriceFetchError::Other(eyre::eyre!(
            "Scryfall returned {}",
            response.status()
        )));
    }

    let fetched_at = Utc::now().to_rfc3339();
    let response: ScryfallCollectionResponse = response
        .json()
        .await
        .map_err(|e| PriceFetchError::Other(e.into()))?;

    Ok(response
        .data
        .into_iter()
        .map(|card| CardPrice {
            source: PRICE_SOURCE_SCRYFALL.to_string(),
            scryfall_id: card.id,
            usd_cents: card.prices.usd.as_deref().and_then(parse_usd_cents),
            usd_foil_cents: card.prices.usd_foil.as_deref().and_then(parse_usd_cents),
            usd_etched_cents: card.prices.usd_etched.as_deref().and_then(parse_usd_cents),
            fetched_at: fetched_at.clone(),
        })
        .collect())
}

pub fn parse_usd_cents(value: &str) -> Option<i64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    let (dollars, cents) = value.split_once('.').unwrap_or((value, ""));
    let dollars = dollars.parse::<i64>().ok()?;
    let mut cents = cents.chars().take(2).collect::<String>();
    while cents.len() < 2 {
        cents.push('0');
    }
    let cents = cents.parse::<i64>().ok()?;
    Some(dollars * 100 + cents)
}

#[cfg(test)]
mod tests {
    use super::parse_usd_cents;

    #[test]
    fn parses_scryfall_price_strings_to_cents() {
        assert_eq!(parse_usd_cents("10.11"), Some(1011));
        assert_eq!(parse_usd_cents("0.5"), Some(50));
        assert_eq!(parse_usd_cents("4"), Some(400));
        assert_eq!(parse_usd_cents(""), None);
    }
}
