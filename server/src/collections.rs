use std::collections::HashMap;

use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::extract::Multipart;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use models::{Card, Collection as ModelCollection};
use persistence::{CollectionCardsParams, PersistenceSystem, PersistenceSystemTrait};
use retrieval::{NamedRetrievalSystem as _, RetrievalSystem, RetrievalSystemTrait};

use crate::{
    ApiError, ErrorPayload, GathersState,
    collections::collections_models::{
        APICardSearchFilters, BulkCollectionSearch, BulkCollectionSearchResult, CardIdentInner,
        CardProxyUpdate, CardToAdd, CardsProxyUpdate, CollectionAddResponse, CollectionCard,
        CollectionCardsQuery, CollectionPriceStats, CollectionRemoveQuery,
        CollectionRemoveResponse, CollectionRename, CollectionSetParent, CollectionsSearchQuery,
        ProxyUpdate, PurchasePrice, PurchasePriceUpdate, ResultCard, ResultCardInner,
    },
    prices::{api_price_from_cache, cached_prices_for_scryfall_ids},
};
use models::CardTrait as _;
pub mod collections_models;

use crate::collections::collections_models::Collection;

const ALL_COLLECTIONS_ID: &str = "__all__";

/// Returns all configured retrieval systems, cloned out of the state lock,
/// keyed by their provider name.
async fn clone_retrieval_systems_by_name(state: &GathersState) -> HashMap<String, RetrievalSystem> {
    let guard = state.0.lock().await;
    [
        guard.mtg.clone(),
        guard.riftbound.clone(),
        guard.pokemon.clone(),
    ]
    .into_iter()
    .flatten()
    .map(|s| (s.name().to_string(), s))
    .collect()
}

fn matches_card_filters(card: &Card, filters: &APICardSearchFilters) -> bool {
    let name_lower: String;
    let set_lower: String;
    let cn: String;

    match card {
        Card::Magic(m) => {
            name_lower = m.name.to_lowercase();
            set_lower = m.set_code.to_lowercase();
            cn = m.collector_number.clone();
        }
        Card::Riftbound(r) => {
            name_lower = r.name.to_lowercase();
            set_lower = r.set_code.to_lowercase();
            cn = r.collector_number.clone();
        }
        Card::Pokemon(p) => {
            name_lower = p.name.to_lowercase();
            set_lower = p.set_code.to_lowercase();
            cn = p.collector_number.clone();
        }
    }

    if let Some(ref v) = filters.name
        && !v.is_empty()
        && !name_lower.contains(&v.to_lowercase())
    {
        return false;
    }
    if let Some(ref v) = filters.set_code
        && !v.is_empty()
        && !set_lower.contains(&v.to_lowercase())
    {
        return false;
    }
    if let Some(ref v) = filters.collector_number
        && !v.is_empty()
        && cn != *v
    {
        return false;
    }

    match card {
        Card::Magic(m) => {
            if let Some(ref v) = filters.artist
                && !v.is_empty()
                && !m.artist.to_lowercase().contains(&v.to_lowercase())
            {
                return false;
            }
            if let Some(ref v) = filters.text
                && !v.is_empty()
                && !m.text.to_lowercase().contains(&v.to_lowercase())
            {
                return false;
            }
            if let Some(ref rarity) = filters.rarity {
                let filter_rarity = models::Rarity::from(rarity.clone());
                if m.rarity != filter_rarity {
                    return false;
                }
            }
            if let Some(ref colors) = filters.color_identities
                && !colors.is_empty()
            {
                let filter_colors: Vec<models::CardColour> = colors
                    .iter()
                    .map(|c| models::CardColour::from(c.clone()))
                    .collect();
                if !filter_colors.iter().all(|c| m.color_identity.contains(c)) {
                    return false;
                }
            }
        }
        Card::Riftbound(r) => {
            if let Some(ref v) = filters.artist
                && !v.is_empty()
                && !r
                    .artists
                    .iter()
                    .any(|a| a.to_lowercase().contains(&v.to_lowercase()))
            {
                return false;
            }
            if let Some(ref v) = filters.text
                && !v.is_empty()
                && !r.text.to_lowercase().contains(&v.to_lowercase())
            {
                return false;
            }
            if let Some(ref domains) = filters.domains
                && !domains.is_empty()
            {
                let filter_domains: Vec<models::riftbound::CardDomain> = domains
                    .iter()
                    .map(|d| models::riftbound::CardDomain::from(d.clone()))
                    .collect();
                if !filter_domains.iter().all(|d| r.domains.contains(d)) {
                    return false;
                }
            }
        }
        Card::Pokemon(p) => {
            if let Some(ref energy) = filters.energy_types
                && !energy.is_empty()
            {
                let filter_energy: Vec<models::pokemon::EnergyType> = energy
                    .iter()
                    .map(|e| models::pokemon::EnergyType::from(e.clone()))
                    .collect();
                if !filter_energy.iter().all(|e| p.energy_types.contains(e)) {
                    return false;
                }
            }
        }
    }

    true
}

fn card_name(card: &Card) -> &str {
    match card {
        Card::Magic(m) => &m.name,
        Card::Riftbound(r) => &r.name,
        Card::Pokemon(p) => &p.name,
    }
}

fn normalized_card_name(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn card_rarity_order(card: &Card) -> u8 {
    match card {
        Card::Magic(m) => match m.rarity {
            models::Rarity::Common => 0,
            models::Rarity::Uncommon => 1,
            models::Rarity::Rare => 2,
            models::Rarity::Mythic => 3,
            _ => 4,
        },
        Card::Riftbound(r) => match r.rarity {
            models::riftbound::RBRarity::Common => 0,
            models::riftbound::RBRarity::Uncommon => 1,
            models::riftbound::RBRarity::Rare => 2,
            models::riftbound::RBRarity::Epic => 3,
            _ => 4,
        },
        Card::Pokemon(p) => match p.rarity {
            models::pokemon::PokemonRarity::Common => 0,
            models::pokemon::PokemonRarity::Uncommon => 1,
            models::pokemon::PokemonRarity::Rare => 2,
            _ => 3,
        },
    }
}

fn sort_collection_cards(
    cards: &mut Vec<&models::CollectionCard>,
    card_data: &HashMap<String, Card>,
    price_cache: &HashMap<String, persistence::CardPrice>,
    sort_by: &Option<crate::collections::collections_models::APISortField>,
    sort_order: &Option<crate::collections::collections_models::APISortOrder>,
) {
    use crate::collections::collections_models::{APISortField, APISortOrder};
    let desc = matches!(sort_order, Some(APISortOrder::Desc));

    cards.sort_by(|a, b| {
        let card_a = card_data.get(&a.uuid);
        let card_b = card_data.get(&b.uuid);
        let ord = match sort_by.as_ref().unwrap_or(&APISortField::Name) {
            APISortField::Name => {
                let na = card_a.map(card_name).unwrap_or("");
                let nb = card_b.map(card_name).unwrap_or("");
                na.cmp(nb)
            }
            APISortField::SetCode => {
                let sa = card_a.map(|c| c.get_set()).unwrap_or_default();
                let sb = card_b.map(|c| c.get_set()).unwrap_or_default();
                sa.cmp(&sb)
            }
            APISortField::CollectorNumber => {
                let ca = card_a.map(|c| c.get_collector_number()).unwrap_or_default();
                let cb = card_b.map(|c| c.get_collector_number()).unwrap_or_default();
                ca.cmp(&cb)
            }
            APISortField::Rarity => {
                let ra = card_a.map(card_rarity_order).unwrap_or(0);
                let rb = card_b.map(card_rarity_order).unwrap_or(0);
                ra.cmp(&rb)
            }
            APISortField::Artist => {
                let aa = match card_a {
                    Some(Card::Magic(m)) => m.artist.as_str(),
                    _ => "",
                };
                let ab = match card_b {
                    Some(Card::Magic(m)) => m.artist.as_str(),
                    _ => "",
                };
                aa.cmp(ab)
            }
            APISortField::TimeAdded => a.time_added.cmp(&b.time_added),
            APISortField::PurchasePrice => {
                let scryfall_id_a = match card_a {
                    Some(Card::Magic(m)) => Some(m.card_identifiers.scryfall_id.as_str()),
                    _ => None,
                };
                let scryfall_id_b = match card_b {
                    Some(Card::Magic(m)) => Some(m.card_identifiers.scryfall_id.as_str()),
                    _ => None,
                };
                let pa = scryfall_id_a
                    .and_then(|id| price_cache.get(id))
                    .and_then(|p| p.usd_cents.or(p.usd_foil_cents))
                    .unwrap_or(0);
                let pb = scryfall_id_b
                    .and_then(|id| price_cache.get(id))
                    .and_then(|p| p.usd_cents.or(p.usd_foil_cents))
                    .unwrap_or(0);
                pa.cmp(&pb)
            }
        };
        if desc { ord.reverse() } else { ord }
    });
}

fn collection_card_response(card: &models::CollectionCard) -> CollectionCard {
    CollectionCard {
        id: card.uuid.clone(),
        quantity: card.quantity,
        foil_quantity: card.foil_quantity,
        collection_id: card.collection.clone(),
        time_added: DateTime::parse_from_rfc3339(&card.time_added)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        provider: card.provider.clone(),
        is_proxy: card.is_proxy,
        purchase_price: purchase_price_response(card),
    }
}

fn purchase_price_response(card: &models::CollectionCard) -> Option<PurchasePrice> {
    Some(PurchasePrice {
        usd_cents: card.purchase_price_cents?,
        source: card
            .purchase_price_source
            .clone()
            .unwrap_or_else(|| "manual".to_string()),
        updated_at: card
            .purchase_price_updated_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now),
    })
}

fn current_unit_price_cents(
    quantity: i32,
    foil_quantity: i32,
    price: Option<&crate::mtg_api::mtg_api_models::APICardPrice>,
) -> Option<i64> {
    let price = price?;
    if quantity > 0 {
        price.usd_cents.or(price.usd_foil_cents)
    } else if foil_quantity > 0 {
        price.usd_foil_cents.or(price.usd_cents)
    } else {
        price.usd_cents.or(price.usd_foil_cents)
    }
}

fn value_for_quantities(
    quantity: i32,
    foil_quantity: i32,
    price: Option<&crate::mtg_api::mtg_api_models::APICardPrice>,
) -> (i64, i64) {
    let Some(price) = price else {
        return (0, 0);
    };
    let mut value = 0;
    let mut priced_copies = 0;

    if quantity > 0
        && let Some(usd_cents) = price.usd_cents
    {
        value += i64::from(quantity) * usd_cents;
        priced_copies += i64::from(quantity);
    }

    if foil_quantity > 0
        && let Some(usd_foil_cents) = price.usd_foil_cents.or(price.usd_cents)
    {
        value += i64::from(foil_quantity) * usd_foil_cents;
        priced_copies += i64::from(foil_quantity);
    }

    (value, priced_copies)
}

fn result_card_from_magic(
    card: &models::MagicCard,
    details: Option<CollectionCard>,
    price: Option<crate::mtg_api::mtg_api_models::APICardPrice>,
) -> ResultCard {
    ResultCard {
        mtg_card: ResultCardInner {
            id: card.id.clone(),
            name: card.name.clone(),
            set_code: card.set_code.clone(),
            card_identifiers: CardIdentInner {
                scryfall_id: card.card_identifiers.scryfall_id.clone(),
            },
            price,
            details,
        },
    }
}

async fn get_collection_cards_for_search(
    state: &GathersState,
    collection_id: Option<&str>,
) -> Result<Vec<models::CollectionCard>, ApiError> {
    let all_params = || CollectionCardsParams {
        offset: 0,
        limit: i64::MAX as usize,
        sort_by: None,
        sort_order: None,
        provider: None,
        providers: vec![],
        proxy_filter: persistence::ProxyFilter::Include,
    };

    let storage = &state.1.lock().await.storage;
    if let Some(collection_id) = collection_id {
        return storage
            .get_cards_in_collection_paginated(&collection_id.to_string(), all_params())
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to get cards from collection. {e}"),
                    }),
                )
            });
    }

    let collections = storage.list_collections(None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorPayload {
                error: format!("Failed to list collections. {e}"),
            }),
        )
    })?;

    let mut cards = Vec::new();
    for collection in collections {
        cards.extend(
            storage
                .get_cards_in_collection_paginated(&collection, all_params())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to get cards from collection. {e}"),
                        }),
                    )
                })?,
        );
    }

    Ok(cards)
}

fn proxy_filter_from_query(value: Option<&str>) -> persistence::ProxyFilter {
    match value.unwrap_or("all").to_ascii_lowercase().as_str() {
        "regular" | "nonproxy" | "non-proxy" | "exclude" => persistence::ProxyFilter::Exclude,
        "proxy" | "only" => persistence::ProxyFilter::Only,
        _ => persistence::ProxyFilter::Include,
    }
}

fn collection_params_from_query(query: CollectionCardsQuery) -> CollectionCardsParams {
    CollectionCardsParams {
        offset: query.offset,
        limit: query.limit.min(1000),
        sort_by: query.sort_by.map(persistence::CollectionSortField::from),
        sort_order: query.sort_order.map(models::filters::SortOrder::from),
        provider: query.provider,
        providers: query
            .providers
            .as_deref()
            .map(|s| {
                s.split(',')
                    .filter(|value| !value.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        proxy_filter: proxy_filter_from_query(query.proxy.as_deref()),
    }
}

fn unpaged_params(params: &CollectionCardsParams) -> CollectionCardsParams {
    CollectionCardsParams {
        offset: 0,
        limit: i64::MAX as usize,
        sort_by: None,
        sort_order: None,
        provider: params.provider.clone(),
        providers: params.providers.clone(),
        proxy_filter: params.proxy_filter,
    }
}

fn sort_collection_rows(cards: &mut [models::CollectionCard], params: &CollectionCardsParams) {
    let desc = matches!(&params.sort_order, Some(models::filters::SortOrder::Desc));
    cards.sort_by(|a, b| {
        let ord = match &params.sort_by {
            Some(persistence::CollectionSortField::Quantity) => a.quantity.cmp(&b.quantity),
            Some(persistence::CollectionSortField::FoilQuantity) => {
                a.foil_quantity.cmp(&b.foil_quantity)
            }
            Some(persistence::CollectionSortField::Provider) => a.provider.cmp(&b.provider),
            Some(persistence::CollectionSortField::PurchasePrice) => {
                let pa = a.purchase_price_cents.unwrap_or(0);
                let pb = b.purchase_price_cents.unwrap_or(0);
                pa.cmp(&pb)
            }
            _ => a.time_added.cmp(&b.time_added),
        };
        if desc { ord.reverse() } else { ord }
    });
}

async fn get_cards_for_collection_scope(
    state: &GathersState,
    collection_id: &str,
    params: CollectionCardsParams,
) -> Result<Vec<models::CollectionCard>, ApiError> {
    let storage = &state.1.lock().await.storage;

    if collection_id == ALL_COLLECTIONS_ID {
        let collections = storage.list_collections(None).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to list collections. {e}"),
                }),
            )
        })?;
        let mut cards = Vec::new();
        let unpaged = unpaged_params(&params);
        for collection in collections {
            cards.extend(
                storage
                    .get_cards_in_collection_paginated(&collection, unpaged.clone())
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorPayload {
                                error: format!("Failed to get cards from collection. {e}"),
                            }),
                        )
                    })?,
            );
        }
        sort_collection_rows(&mut cards, &params);
        return Ok(cards
            .into_iter()
            .skip(params.offset)
            .take(params.limit)
            .collect());
    }

    let children: Vec<String> = storage
        .list_collection_details(None)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to list collections. {e}"),
                }),
            )
        })?
        .into_iter()
        .filter(|c| c.parent.as_deref() == Some(collection_id))
        .map(|c| c.id)
        .collect();

    if children.is_empty() {
        return storage
            .get_cards_in_collection_paginated(&collection_id.to_string(), params)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to get cards from collection. {e}"),
                    }),
                )
            });
    }

    let unpaged = unpaged_params(&params);
    let mut cards = Vec::new();
    for child in children {
        cards.extend(
            storage
                .get_cards_in_collection_paginated(&child, unpaged.clone())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to get cards from collection. {e}"),
                        }),
                    )
                })?,
        );
    }
    sort_collection_rows(&mut cards, &params);
    Ok(cards
        .into_iter()
        .skip(params.offset)
        .take(params.limit)
        .collect())
}

async fn search_owned_magic_cards(
    state: &GathersState,
    query: &CollectionsSearchQuery,
    filters: &APICardSearchFilters,
) -> Result<Vec<ResultCard>, ApiError> {
    let retrieval_systems = clone_retrieval_systems_by_name(state).await;
    let collection_cards =
        get_collection_cards_for_search(state, query.collection.as_deref()).await?;

    let mut by_provider: HashMap<String, Vec<models::CollectionCard>> = HashMap::new();
    for card in collection_cards {
        by_provider
            .entry(card.provider.clone())
            .or_default()
            .push(card);
    }

    let mut card_data: HashMap<String, Card> = HashMap::new();
    for (provider, cards) in &by_provider {
        if let Some(retrieval) = retrieval_systems.get(provider) {
            let ids: Vec<String> = cards.iter().map(|c| c.uuid.clone()).collect();
            if let Ok(data) = retrieval.get_cards_by_ids(ids).await {
                card_data.extend(data);
            }
        }
    }

    let mut matched: Vec<&models::CollectionCard> = by_provider
        .values()
        .flatten()
        .filter(|cc| {
            card_data
                .get(&cc.uuid)
                .map(|card| matches!(card, Card::Magic(_)) && matches_card_filters(card, filters))
                .unwrap_or(false)
        })
        .collect();

    // Fetch prices for all matched cards before sorting so price-based sort
    // uses current market price rather than the stored purchase price.
    let sort_price_cache = if matches!(
        filters.sort_by,
        Some(crate::collections::collections_models::APISortField::PurchasePrice)
    ) {
        cached_prices_for_scryfall_ids(
            state,
            matched.iter().filter_map(|cc| match card_data.get(&cc.uuid) {
                Some(Card::Magic(card)) => Some(card.card_identifiers.scryfall_id.clone()),
                _ => None,
            }),
            1,
        )
        .await
        .unwrap_or_default()
    } else {
        HashMap::new()
    };

    sort_collection_cards(
        &mut matched,
        &card_data,
        &sort_price_cache,
        &filters.sort_by,
        &filters.sort_order,
    );

    let page: Vec<&models::CollectionCard> = matched
        .into_iter()
        .skip(query.offset)
        .take(query.page_size.min(1000))
        .collect();
    let price_cache = cached_prices_for_scryfall_ids(
        state,
        page.iter().filter_map(|cc| match card_data.get(&cc.uuid) {
            Some(Card::Magic(card)) => Some(card.card_identifiers.scryfall_id.clone()),
            _ => None,
        }),
        1,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorPayload {
                error: format!("Failed to read card prices. {e}"),
            }),
        )
    })?;

    Ok(page
        .into_iter()
        .filter_map(|cc| match card_data.get(&cc.uuid) {
            Some(Card::Magic(card)) => Some(result_card_from_magic(
                card,
                Some(collection_card_response(cc)),
                price_cache
                    .get(&card.card_identifiers.scryfall_id)
                    .map(api_price_from_cache),
            )),
            _ => None,
        })
        .collect())
}

async fn price_stats_for_collection_cards(
    state: &GathersState,
    collection_id: Option<String>,
    cards: Vec<models::CollectionCard>,
) -> Result<CollectionPriceStats, ApiError> {
    let retrieval_systems = clone_retrieval_systems_by_name(state).await;
    let mut by_provider: HashMap<String, Vec<models::CollectionCard>> = HashMap::new();
    for card in &cards {
        by_provider
            .entry(card.provider.clone())
            .or_default()
            .push(card.clone());
    }

    let mut card_data: HashMap<String, Card> = HashMap::new();
    for (provider, cards) in &by_provider {
        if let Some(retrieval) = retrieval_systems.get(provider) {
            let ids: Vec<String> = cards.iter().map(|c| c.uuid.clone()).collect();
            if let Ok(data) = retrieval.get_cards_by_ids(ids).await {
                card_data.extend(data);
            }
        }
    }

    let price_cache = cached_prices_for_scryfall_ids(
        state,
        card_data.values().filter_map(|card| match card {
            Card::Magic(card) => Some(card.card_identifiers.scryfall_id.clone()),
            _ => None,
        }),
        1,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorPayload {
                error: format!("Failed to read card prices. {e}"),
            }),
        )
    })?;

    let mut copy_count = 0;
    let mut priced_copy_count = 0;
    let mut baseline_copy_count = 0;
    let mut total_value_cents = 0;
    let mut tracked_current_value_cents = 0;
    let mut purchase_value_cents = 0;
    let mut proxy_card_count = 0;
    let mut proxy_copy_count = 0;
    let mut proxy_priced_copy_count = 0;
    let mut proxy_total_value_cents = 0;

    for card in &cards {
        let row_copy_count = i64::from(card.quantity.max(0) + card.foil_quantity.max(0));
        let api_price = card_data
            .get(&card.uuid)
            .and_then(|card_data| match card_data {
                Card::Magic(card) => price_cache
                    .get(&card.card_identifiers.scryfall_id)
                    .map(api_price_from_cache),
                _ => None,
            });
        let (current_value, current_priced_copies) =
            value_for_quantities(card.quantity, card.foil_quantity, api_price.as_ref());

        if card.is_proxy {
            proxy_card_count += 1;
            proxy_copy_count += row_copy_count;
            proxy_priced_copy_count += current_priced_copies;
            proxy_total_value_cents += current_value;
        } else {
            copy_count += row_copy_count;
            total_value_cents += current_value;
            priced_copy_count += current_priced_copies;

            if let Some(purchase_price_cents) = card.purchase_price_cents {
                baseline_copy_count += row_copy_count;
                purchase_value_cents += purchase_price_cents * row_copy_count;
                tracked_current_value_cents += current_value;
            }
        }
    }

    let change_cents =
        (purchase_value_cents > 0).then_some(tracked_current_value_cents - purchase_value_cents);
    let change_percent =
        change_cents.map(|change| (change as f64 / purchase_value_cents as f64) * 100.0);

    Ok(CollectionPriceStats {
        collection_id,
        card_count: cards.len(),
        copy_count,
        priced_copy_count,
        baseline_copy_count,
        total_value_cents,
        tracked_current_value_cents,
        purchase_value_cents,
        change_cents,
        change_percent,
        proxy_card_count,
        proxy_copy_count,
        proxy_priced_copy_count,
        proxy_total_value_cents,
    })
}

pub fn collection_routes() -> ApiRouter<GathersState> {
    async fn list(State(state): State<GathersState>) -> Result<Json<Vec<Collection>>, ApiError> {
        let storage = &state.1.lock().await.storage;

        match storage.list_collection_details(None).await {
            Ok(collections) => Ok(Json(
                collections
                    .iter()
                    .map(|c| Collection {
                        id: c.id.clone(),
                        can_remove: c.can_remove,
                        is_proxy: c.is_proxy,
                        parent: c.parent.clone(),
                    })
                    .collect(),
            )),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to list collections. {e}"),
                }),
            )),
        }
    }

    async fn add(
        State(state): State<GathersState>,
        Json(input): Json<Collection>,
    ) -> Result<Json<CollectionAddResponse>, ApiError> {
        validate_collection_name(&input.id)?;

        let storage = &mut state.1.lock().await.storage;

        let collection_id = storage.add_collection(input.id.clone()).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to add collection. {e}"),
                }),
            )
        })?;

        if input.is_proxy {
            storage
                .set_collection_proxy(&collection_id, true)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to mark collection as proxy. {e}"),
                        }),
                    )
                })?;
        }

        if let Some(ref parent_id) = input.parent {
            storage
                .set_collection_parent(&collection_id, Some(parent_id.clone()))
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to set collection parent. {e}"),
                        }),
                    )
                })?;
        }

        Ok(Json(CollectionAddResponse {
            id: collection_id,
            name: input.id,
        }))
    }

    async fn remove(
        State(state): State<GathersState>,
        Path(id): Path<String>,
        Query(query): Query<CollectionRemoveQuery>,
    ) -> Result<Json<CollectionRemoveResponse>, ApiError> {
        let storage = &mut state.1.lock().await.storage;

        let reparent_children_to = query.reparent_children_to.filter(|v| !v.trim().is_empty());
        let children: Vec<ModelCollection> = storage
            .list_collection_details(None)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to list collections. {e}"),
                    }),
                )
            })?
            .into_iter()
            .filter(|c| c.parent.as_deref() == Some(id.as_str()))
            .collect();

        for child in children {
            storage
                .set_collection_parent(&child.id, reparent_children_to.clone())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to reparent child collection. {e}"),
                        }),
                    )
                })?;
        }

        let move_to = query
            .keep_cards_in_collection
            .or(query.move_to)
            .filter(|value| !value.trim().is_empty());
        match storage.remove_collection(&id, move_to).await {
            Ok(message) => Ok(Json(CollectionRemoveResponse { message })),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to remove collection. {e}"),
                }),
            )),
        }
    }

    async fn rename(
        State(state): State<GathersState>,
        Path(id): Path<String>,
        Json(input): Json<CollectionRename>,
    ) -> Result<Json<Collection>, ApiError> {
        validate_collection_name(&input.id)?;
        let storage = &mut state.1.lock().await.storage;
        match storage.rename_collection(&id, input.id).await {
            Ok(collection) => Ok(Json(Collection {
                id: collection.id,
                can_remove: collection.can_remove,
                is_proxy: collection.is_proxy,
                parent: collection.parent,
            })),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to rename collection. {e}"),
                }),
            )),
        }
    }

    async fn collection_proxy_update(
        State(state): State<GathersState>,
        Path(id): Path<String>,
        Json(input): Json<ProxyUpdate>,
    ) -> Result<Json<Collection>, ApiError> {
        let storage = &mut state.1.lock().await.storage;
        match storage.set_collection_proxy(&id, input.is_proxy).await {
            Ok(collection) => Ok(Json(Collection {
                id: collection.id,
                can_remove: collection.can_remove,
                is_proxy: collection.is_proxy,
                parent: collection.parent,
            })),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to update collection proxy status. {e}"),
                }),
            )),
        }
    }

    #[axum::debug_handler]
    async fn move_to(
        State(state): State<GathersState>,
        Path(to_collection_id): Path<String>,
        Json(input): Json<Vec<CollectionCard>>,
    ) -> Result<Json<()>, ApiError> {
        let storage = &mut state.1.lock().await.storage;

        let cards: Vec<models::CollectionCard> = input.iter().map(|card| card.into()).collect();
        match storage
            .move_cards_between_collections(&cards, to_collection_id)
            .await
        {
            Ok(_) => Ok(Json(())),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to move cards. {e}"),
                }),
            )),
        }
    }

    fn validate_collection_name(name: &str) -> Result<(), ApiError> {
        if name.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Collection name cannot be empty".to_string(),
                }),
            ));
        }
        if name == ALL_COLLECTIONS_ID {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Collection name is reserved".to_string(),
                }),
            ));
        }
        if name.len() > 255 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Collection name too long (max 255 characters)".to_string(),
                }),
            ));
        }
        if name.chars().any(|c| c.is_control()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Collection name contains invalid characters".to_string(),
                }),
            ));
        }
        Ok(())
    }

    async fn validate_collection(
        storage: &mut PersistenceSystem,
        collection_id: &String,
    ) -> Result<(), Json<ErrorPayload>> {
        let collections = match storage.list_collections(None).await {
            Ok(collections) => collections,
            Err(e) => {
                return Err(Json(ErrorPayload {
                    error: format!("Failed to verify collection. {e}"),
                }));
            }
        };

        if !collections.contains(collection_id) {
            return Err(Json(ErrorPayload {
                error: "Collection not found".to_string(),
            }));
        }
        Ok(())
    }

    async fn mutate_card_quantities(
        storage: &mut PersistenceSystem,
        collection_id: &str,
        uuid: String,
        quantity: i32,
        foil_quantity: i32,
        provider: String,
        purchase_price_cents: Option<i64>,
        purchase_price_source: Option<&str>,
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();

        match storage
            .add_card_to_collection(
                &collection_id.to_string(),
                &uuid,
                quantity,
                foil_quantity,
                &now_str,
                &provider,
            )
            .await
        {
            Ok(mut card) => {
                if card.purchase_price_cents.is_none()
                    && let Some(purchase_price_cents) = purchase_price_cents
                {
                    card = storage
                        .set_card_purchase_price(
                            &collection_id.to_string(),
                            &uuid,
                            Some(purchase_price_cents),
                            purchase_price_source,
                            &now_str,
                        )
                        .await
                        .map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorPayload {
                                    error: format!("Failed to set purchase price. {e}"),
                                }),
                            )
                        })?;
                }
                Ok(Json(vec![collection_card_response(&card)]))
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to change card quantity in collection. {e}"),
                }),
            )),
        }
    }

    async fn cards_add(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Json(input): Json<CardToAdd>,
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        // Identify the provider by finding which configured system has this card.
        let systems = clone_retrieval_systems_by_name(&state).await;
        let card_ids = vec![input.id.clone()];
        let mut provider = String::new();
        let mut scryfall_id = None;
        for (name, system) in &systems {
            if let Ok(found) = system.get_cards_by_ids(card_ids.clone()).await
                && !found.is_empty()
            {
                provider = name.clone();
                scryfall_id = found.values().find_map(|card| match card {
                    Card::Magic(card) => Some(card.card_identifiers.scryfall_id.clone()),
                    _ => None,
                });
                break;
            }
        }

        let (purchase_price_cents, purchase_price_source) = if let Some(purchase_price_cents) =
            input.purchase_price_cents
        {
            (Some(purchase_price_cents), Some("manual"))
        } else if let Some(scryfall_id) = scryfall_id {
            let price_cache = cached_prices_for_scryfall_ids(&state, [scryfall_id.clone()], 1)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to read card prices. {e}"),
                        }),
                    )
                })?;
            let api_price = price_cache.get(&scryfall_id).map(api_price_from_cache);
            (
                current_unit_price_cents(input.quantity, input.foil_quantity, api_price.as_ref()),
                Some("market_at_add"),
            )
        } else {
            (None, None)
        };

        let storage = &mut state.1.lock().await.storage;

        if let Err(e) = validate_collection(storage, &collection_id).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e));
        };

        let is_parent = storage
            .list_collection_details(None)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to list collections. {e}"),
                    }),
                )
            })?
            .iter()
            .any(|c| c.parent.as_deref() == Some(collection_id.as_str()));
        if is_parent {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Cards cannot be added directly to a parent collection".to_string(),
                }),
            ));
        }

        mutate_card_quantities(
            storage,
            &collection_id,
            input.id,
            input.quantity,
            input.foil_quantity,
            provider,
            purchase_price_cents,
            purchase_price_source,
        )
        .await
    }

    async fn cards_remove(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Json(input): Json<CardToAdd>,
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        let storage = &mut state.1.lock().await.storage;

        if let Err(e) = validate_collection(storage, &collection_id).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e));
        };

        let neg_quantity = input.quantity.checked_neg().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Invalid quantity".to_string(),
                }),
            )
        })?;
        let neg_foil_quantity = input.foil_quantity.checked_neg().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Invalid foil quantity".to_string(),
                }),
            )
        })?;

        mutate_card_quantities(
            storage,
            &collection_id,
            input.id,
            neg_quantity,
            neg_foil_quantity,
            "".to_string(),
            None,
            None,
        )
        .await
    }

    async fn cards_get(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Query(query): Query<CollectionCardsQuery>,
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        let collection_params = collection_params_from_query(query);
        let cards =
            get_cards_for_collection_scope(&state, &collection_id, collection_params).await?;

        let response_cards = cards
            .into_iter()
            .map(|card| collection_card_response(&card))
            .collect();

        Ok(Json(response_cards))
    }

    async fn collection_cards_count(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Query(query): Query<CollectionCardsQuery>,
    ) -> Result<Json<usize>, ApiError> {
        let providers: Vec<String> = if let Some(p) = query.provider {
            vec![p]
        } else {
            query
                .providers
                .as_deref()
                .map(|s| s.split(',').map(str::to_string).collect())
                .unwrap_or_default()
        };
        let proxy_filter = proxy_filter_from_query(query.proxy.as_deref());

        let params = CollectionCardsParams {
            offset: 0,
            limit: i64::MAX as usize,
            sort_by: None,
            sort_order: None,
            provider: None,
            providers,
            proxy_filter,
        };
        get_cards_for_collection_scope(&state, &collection_id, params)
            .await
            .map(|cards| Json(cards.len()))
    }

    async fn collection_cards_stats(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
    ) -> Result<Json<CollectionPriceStats>, ApiError> {
        if collection_id == ALL_COLLECTIONS_ID {
            return all_collections_stats(State(state)).await;
        }

        let cards = get_cards_for_collection_scope(
            &state,
            &collection_id,
            CollectionCardsParams {
                offset: 0,
                limit: i64::MAX as usize,
                sort_by: None,
                sort_order: None,
                provider: None,
                providers: vec![],
                proxy_filter: persistence::ProxyFilter::Include,
            },
        )
        .await?;

        price_stats_for_collection_cards(&state, Some(collection_id), cards)
            .await
            .map(Json)
    }

    async fn all_collections_stats(
        State(state): State<GathersState>,
    ) -> Result<Json<CollectionPriceStats>, ApiError> {
        let cards = {
            let storage = &state.1.lock().await.storage;
            let collections = storage.list_collections(None).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to list collections. {e}"),
                    }),
                )
            })?;

            let mut cards = Vec::new();
            for collection in collections {
                cards.extend(
                    storage
                        .get_cards_in_collection_paginated(
                            &collection,
                            CollectionCardsParams {
                                offset: 0,
                                limit: i64::MAX as usize,
                                sort_by: None,
                                sort_order: None,
                                provider: None,
                                providers: vec![],
                                proxy_filter: persistence::ProxyFilter::Include,
                            },
                        )
                        .await
                        .map_err(|e| {
                            (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                Json(ErrorPayload {
                                    error: format!("Failed to get cards from collection. {e}"),
                                }),
                            )
                        })?,
                );
            }
            cards
        };

        price_stats_for_collection_cards(&state, None, cards)
            .await
            .map(Json)
    }

    async fn purchase_price_update(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Json(input): Json<PurchasePriceUpdate>,
    ) -> Result<Json<CollectionCard>, ApiError> {
        let now = chrono::Utc::now().to_rfc3339();
        let card = state
            .1
            .lock()
            .await
            .storage
            .set_card_purchase_price(
                &collection_id,
                &input.id,
                input.purchase_price_cents,
                input.purchase_price_cents.map(|_| "manual"),
                &now,
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to set purchase price. {e}"),
                    }),
                )
            })?;

        Ok(Json(collection_card_response(&card)))
    }

    async fn card_proxy_update(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Json(input): Json<CardProxyUpdate>,
    ) -> Result<Json<CollectionCard>, ApiError> {
        let card = state
            .1
            .lock()
            .await
            .storage
            .set_card_proxy(&collection_id, &input.id, input.is_proxy)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to set card proxy status. {e}"),
                    }),
                )
            })?;

        Ok(Json(collection_card_response(&card)))
    }

    async fn cards_proxy_update(
        State(state): State<GathersState>,
        Json(input): Json<CardsProxyUpdate>,
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        let storage = &mut state.1.lock().await.storage;
        let mut updated = Vec::new();
        for card in input.cards {
            let card = storage
                .set_card_proxy(&card.collection_id, &card.id, input.is_proxy)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to set card proxy status. {e}"),
                        }),
                    )
                })?;
            updated.push(collection_card_response(&card));
        }

        Ok(Json(updated))
    }

    async fn search_temp(
        State(state): State<GathersState>,
        Query(query): Query<CollectionsSearchQuery>,
        Json(input): Json<APICardSearchFilters>,
    ) -> Result<Json<Vec<ResultCard>>, ApiError> {
        if query.skip_not_owned || query.collection.is_some() {
            return search_owned_magic_cards(&state, &query, &input)
                .await
                .map(Json);
        }

        let guard = state.0.lock().await;
        let ret = guard.require_mtg()?;

        let result = ret
            .search_cards(
                input.into(),
                query.offset.into(),
                query.page_size.min(1000).into(),
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to search cards. {e}"),
                    }),
                )
            })?;
        drop(guard);

        let price_cache = cached_prices_for_scryfall_ids(
            &state,
            result.iter().filter_map(|c| match c {
                Card::Magic(m) => Some(m.card_identifiers.scryfall_id.clone()),
                _ => None,
            }),
            0,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to read card prices. {e}"),
                }),
            )
        })?;

        Ok(Json(
            result
                .iter()
                .filter_map(|c| match c {
                    Card::Magic(m) => Some(result_card_from_magic(
                        m,
                        None,
                        price_cache
                            .get(&m.card_identifiers.scryfall_id)
                            .map(api_price_from_cache),
                    )),
                    _ => None,
                })
                .collect(),
        ))
    }

    async fn bulk_search(
        State(state): State<GathersState>,
        Query(query): Query<CollectionsSearchQuery>,
        Json(input): Json<BulkCollectionSearch>,
    ) -> Result<Json<Vec<BulkCollectionSearchResult>>, ApiError> {
        let mut requests: Vec<(String, String, i32)> = Vec::new();
        let mut request_indexes: HashMap<String, usize> = HashMap::new();

        for card in input.cards {
            let key = normalized_card_name(&card.name);
            if key.is_empty() || card.quantity <= 0 {
                continue;
            }

            if let Some(index) = request_indexes.get(&key).copied() {
                requests[index].2 += card.quantity;
            } else {
                request_indexes.insert(key.clone(), requests.len());
                requests.push((key, card.name.trim().to_string(), card.quantity));
            }
        }

        if requests.is_empty() {
            return Ok(Json(Vec::new()));
        }

        let collection_cards =
            get_collection_cards_for_search(&state, query.collection.as_deref()).await?;
        let retrieval_systems = clone_retrieval_systems_by_name(&state).await;

        let mut by_provider: HashMap<String, Vec<models::CollectionCard>> = HashMap::new();
        for card in collection_cards {
            by_provider
                .entry(card.provider.clone())
                .or_default()
                .push(card);
        }

        let mut card_data: HashMap<String, Card> = HashMap::new();
        for (provider, cards) in &by_provider {
            if let Some(retrieval) = retrieval_systems.get(provider) {
                let ids: Vec<String> = cards.iter().map(|c| c.uuid.clone()).collect();
                if let Ok(data) = retrieval.get_cards_by_ids(ids).await {
                    card_data.extend(data);
                }
            }
        }

        let mut matched: Vec<&models::CollectionCard> = by_provider
            .values()
            .flatten()
            .filter(|cc| {
                card_data
                    .get(&cc.uuid)
                    .map(|card| {
                        matches!(card, Card::Magic(_))
                            && request_indexes.contains_key(&normalized_card_name(card_name(card)))
                    })
                    .unwrap_or(false)
            })
            .collect();

        sort_collection_cards(
            &mut matched,
            &card_data,
            &HashMap::new(),
            &Some(crate::collections::collections_models::APISortField::Name),
            &Some(crate::collections::collections_models::APISortOrder::Asc),
        );

        let price_cache = cached_prices_for_scryfall_ids(
            &state,
            matched
                .iter()
                .filter_map(|cc| match card_data.get(&cc.uuid) {
                    Some(Card::Magic(card)) => Some(card.card_identifiers.scryfall_id.clone()),
                    _ => None,
                }),
            1,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to read card prices. {e}"),
                }),
            )
        })?;

        let mut owned_by_name: HashMap<String, i32> = HashMap::new();
        let mut matches_by_name: HashMap<String, Vec<ResultCard>> = HashMap::new();

        for cc in matched {
            if let Some(Card::Magic(card)) = card_data.get(&cc.uuid) {
                let key = normalized_card_name(&card.name);
                *owned_by_name.entry(key.clone()).or_insert(0) += cc.quantity + cc.foil_quantity;
                matches_by_name
                    .entry(key)
                    .or_default()
                    .push(result_card_from_magic(
                        card,
                        Some(collection_card_response(cc)),
                        price_cache
                            .get(&card.card_identifiers.scryfall_id)
                            .map(api_price_from_cache),
                    ));
            }
        }

        Ok(Json(
            requests
                .into_iter()
                .map(|(key, name, requested_quantity)| {
                    let owned_quantity = owned_by_name.get(&key).copied().unwrap_or(0);
                    BulkCollectionSearchResult {
                        name,
                        requested_quantity,
                        owned_quantity,
                        needed_quantity: (requested_quantity - owned_quantity).max(0),
                        matches: matches_by_name.remove(&key).unwrap_or_default(),
                    }
                })
                .collect(),
        ))
    }

    async fn export(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
    ) -> Result<Response, ApiError> {
        let retrievals: Vec<RetrievalSystem> = {
            let guard = state.0.lock().await;
            [
                guard.mtg.clone(),
                guard.riftbound.clone(),
                guard.pokemon.clone(),
            ]
            .into_iter()
            .flatten()
            .collect()
        };

        let csv = state
            .1
            .lock()
            .await
            .storage
            .export_collection(&collection_id, &retrievals)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Export failed: {e}"),
                    }),
                )
            })?;

        let safe_filename: String = collection_id
            .chars()
            .filter(|c| *c != '"' && *c != '\\' && !c.is_control())
            .collect();
        let filename = format!("attachment; filename=\"{safe_filename}.csv\"");
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"))
            .header(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&filename).unwrap_or_else(|_| {
                    HeaderValue::from_static("attachment; filename=\"collection.csv\"")
                }),
            )
            .body(axum::body::Body::from(csv))
            .unwrap())
    }

    async fn import(
        State(state): State<GathersState>,
        mut multipart: Multipart,
    ) -> Result<Json<()>, ApiError> {
        let mut file_bytes: Option<Vec<u8>> = None;
        let mut csv_text: Option<String> = None;
        let mut collection_name: Option<String> = None;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: format!("Failed to read multipart field: {e}"),
                }),
            )
        })? {
            match field.name() {
                Some("file") => {
                    file_bytes = Some(
                        field
                            .bytes()
                            .await
                            .map_err(|e| {
                                (
                                    StatusCode::BAD_REQUEST,
                                    Json(ErrorPayload {
                                        error: format!("Failed to read file bytes: {e}"),
                                    }),
                                )
                            })?
                            .to_vec(),
                    );
                }
                Some("text") | Some("csvText") | Some("csv_text") => {
                    csv_text = Some(field.text().await.map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorPayload {
                                error: format!("Failed to read text field: {e}"),
                            }),
                        )
                    })?);
                }
                Some("collection") => {
                    collection_name = Some(field.text().await.map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorPayload {
                                error: format!("Failed to read collection field: {e}"),
                            }),
                        )
                    })?);
                }
                _ => {}
            }
        }

        let csv_text = csv_text.filter(|text| !text.trim().is_empty());
        if file_bytes.is_none() && csv_text.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "No file or text provided".to_string(),
                }),
            ));
        }

        let collection_name = collection_name.unwrap_or_else(|| "New Collection".to_string());
        validate_collection_name(&collection_name)?;

        let retrievals: Vec<RetrievalSystem> = {
            let guard = state.0.lock().await;
            [
                guard.mtg.clone(),
                guard.riftbound.clone(),
                guard.pokemon.clone(),
            ]
            .into_iter()
            .flatten()
            .collect()
        };

        let import_log: persistence::ImportLog =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        let mut storage = state.1.lock().await;
        let result = match (file_bytes, csv_text) {
            (Some(bytes), _) => {
                let csv_content = String::from_utf8_lossy(&bytes).into_owned();
                crate::save_import_csv(&collection_name, &csv_content);

                let mut tmp = tempfile::NamedTempFile::new().map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to create temp file: {e}"),
                        }),
                    )
                })?;
                std::io::Write::write_all(&mut tmp, &bytes).map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: format!("Failed to write temp file: {e}"),
                        }),
                    )
                })?;
                let tmp_path = tmp.path().to_string_lossy().to_string();

                storage
                    .storage
                    .import_csv(tmp_path, collection_name, &retrievals, None, Some(import_log.clone()))
                    .await
            }
            (None, Some(text)) => {
                crate::save_import_csv(&collection_name, &text);
                storage
                    .storage
                    .import_csv_text(&text, collection_name, &retrievals, None, Some(import_log.clone()))
                    .await
            }
            (None, None) => unreachable!("validated import source above"),
        };

        let log_msgs = import_log.lock().map(|v| v.clone()).unwrap_or_default();
        crate::push_debug_logs(log_msgs);

        result.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Import failed: {e}"),
                }),
            )
        })?;

        Ok(Json(()))
    }

    async fn collection_cards_search(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Query(query): Query<CollectionCardsQuery>,
        Json(filters): Json<APICardSearchFilters>,
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        let retrieval_systems = clone_retrieval_systems_by_name(&state).await;
        let offset = query.offset;
        let limit = query.limit;
        let params = collection_params_from_query(query);
        let collection_cards =
            get_cards_for_collection_scope(&state, &collection_id, unpaged_params(&params)).await?;

        let mut by_provider: HashMap<String, Vec<models::CollectionCard>> = HashMap::new();
        for card in collection_cards {
            by_provider
                .entry(card.provider.clone())
                .or_default()
                .push(card);
        }

        let mut card_data: HashMap<String, Card> = HashMap::new();
        for (provider, cards) in &by_provider {
            if let Some(retrieval) = retrieval_systems.get(provider) {
                let ids: Vec<String> = cards.iter().map(|c| c.uuid.clone()).collect();
                if let Ok(data) = retrieval.get_cards_by_ids(ids).await {
                    card_data.extend(data);
                }
            }
        }

        let mut matched: Vec<&models::CollectionCard> = by_provider
            .values()
            .flatten()
            .filter(|cc| {
                card_data
                    .get(&cc.uuid)
                    .map(|card| matches_card_filters(card, &filters))
                    .unwrap_or(false)
            })
            .collect();

        sort_collection_cards(
            &mut matched,
            &card_data,
            &HashMap::new(),
            &filters.sort_by,
            &filters.sort_order,
        );

        let page: Vec<CollectionCard> = matched
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(collection_card_response)
            .collect();

        Ok(Json(page))
    }

    async fn collection_cards_search_count(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Query(query): Query<CollectionCardsQuery>,
        Json(filters): Json<APICardSearchFilters>,
    ) -> Result<Json<usize>, ApiError> {
        let retrieval_systems = clone_retrieval_systems_by_name(&state).await;
        let params = collection_params_from_query(query);
        let collection_cards =
            get_cards_for_collection_scope(&state, &collection_id, unpaged_params(&params)).await?;

        let mut by_provider: HashMap<String, Vec<models::CollectionCard>> = HashMap::new();
        for card in collection_cards {
            by_provider
                .entry(card.provider.clone())
                .or_default()
                .push(card);
        }

        let mut card_data: HashMap<String, Card> = HashMap::new();
        for (provider, cards) in &by_provider {
            if let Some(retrieval) = retrieval_systems.get(provider) {
                let ids: Vec<String> = cards.iter().map(|c| c.uuid.clone()).collect();
                if let Ok(data) = retrieval.get_cards_by_ids(ids).await {
                    card_data.extend(data);
                }
            }
        }

        let count = by_provider
            .values()
            .flatten()
            .filter(|cc| {
                card_data
                    .get(&cc.uuid)
                    .map(|card| matches_card_filters(card, &filters))
                    .unwrap_or(false)
            })
            .count();

        Ok(Json(count))
    }

    async fn set_parent(
        State(state): State<GathersState>,
        Path(id): Path<String>,
        Json(input): Json<CollectionSetParent>,
    ) -> Result<Json<Collection>, ApiError> {
        let storage = &mut state.1.lock().await.storage;

        if let Some(ref parent_id) = input.parent {
            if parent_id == &id {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorPayload {
                        error: "A collection cannot be its own parent".to_string(),
                    }),
                ));
            }
            let all = storage.list_collection_details(None).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to list collections. {e}"),
                    }),
                )
            })?;
            let parent = all.iter().find(|c| &c.id == parent_id).ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorPayload {
                        error: "Parent collection not found".to_string(),
                    }),
                )
            })?;
            if parent.parent.is_some() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorPayload {
                        error: "Target parent is itself a child collection".to_string(),
                    }),
                ));
            }
            let target_is_parent = all.iter().any(|c| c.parent.as_deref() == Some(id.as_str()));
            if target_is_parent {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ErrorPayload {
                        error: "A parent collection cannot be assigned a parent".to_string(),
                    }),
                ));
            }
        }

        match storage.set_collection_parent(&id, input.parent).await {
            Ok(collection) => Ok(Json(Collection {
                id: collection.id,
                can_remove: collection.can_remove,
                is_proxy: collection.is_proxy,
                parent: collection.parent,
            })),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to set collection parent. {e}"),
                }),
            )),
        }
    }

    ApiRouter::new()
        .api_route("/list", get(list))
        .api_route("/add", post(add))
        .api_route("/rename/{id}", post(rename))
        .api_route("/proxy/{id}", post(collection_proxy_update))
        .api_route("/remove/{id}", post(remove))
        .api_route("/set-parent/{id}", post(set_parent))
        .api_route("/move/{id}", post(move_to))
        .api_route("/stats", get(all_collections_stats))
        .api_route("/cards/{id}/list", get(cards_get))
        .api_route("/cards/{id}/count", get(collection_cards_count))
        .api_route("/cards/{id}/stats", get(collection_cards_stats))
        .api_route("/cards/{id}/search", post(collection_cards_search))
        .api_route(
            "/cards/{id}/search/count",
            post(collection_cards_search_count),
        )
        .api_route("/search", post(search_temp))
        .api_route("/bulk-search", post(bulk_search))
        .api_route("/cards/{id}/add", post(cards_add))
        .api_route("/cards/{id}/delete", post(cards_remove))
        .api_route("/cards/{id}/purchase-price", post(purchase_price_update))
        .api_route("/cards/{id}/proxy", post(card_proxy_update))
        .api_route("/cards/proxy", post(cards_proxy_update))
        .route("/import", axum::routing::post(import))
        .route("/export/{id}", axum::routing::get(export))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode},
    };
    use persistence::{PersistenceSystemTrait, SQLitePersistenceSystem};
    use retrieval::MagicSQLiteRetrievalSystem;
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    use super::*;

    fn test_state() -> GathersState {
        let retrieval = RetrievalSystem::MagicSQLiteRetrievalSystem(
            MagicSQLiteRetrievalSystem::new(None).unwrap(),
        );
        let storage = PersistenceSystem::SQLitePersistenceSystem(
            SQLitePersistenceSystem::new(true, None).unwrap(),
        );
        (
            Arc::new(Mutex::new(crate::RetrievalState {
                mtg: Some(retrieval),
                riftbound: None,
                pokemon: None,
                mtg_system_type: Some(crate::Systems::Sql),
                mtg_db_path: None,
                riftbound_db_path: None,
                pokemon_db_path: None,
                downloading: HashMap::new(),
            })),
            Arc::new(Mutex::new(crate::StorageState {
                storage,
                _storage_db_path: None,
            })),
        )
    }

    #[tokio::test]
    async fn search_owned_magic_cards_includes_collection_details() {
        let state = test_state();
        let card_id = "0005d268-3fd0-5424-bc6b-573ecd713aa1".to_string();
        let a1 = "A1".to_string();
        let default = "Default".to_string();
        let time_added = "2024-01-01T00:00:00Z";

        {
            let storage = &mut state.1.lock().await.storage;
            storage.add_collection(a1.clone()).await.unwrap();
            storage
                .add_card_to_collection(&a1, &card_id, 1, 0, time_added, "MagicSQLite")
                .await
                .unwrap();
            storage
                .add_card_to_collection(&default, &card_id, 2, 0, time_added, "MagicSQLite")
                .await
                .unwrap();
        }

        let response = collection_routes()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/search?skipNotOwned=true&pageSize=24&offset=0")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"War Priest"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let all_results: Vec<ResultCard> = serde_json::from_slice(&body).unwrap();
        let collections: std::collections::HashSet<String> = all_results
            .iter()
            .filter_map(|card| {
                card.mtg_card
                    .details
                    .as_ref()
                    .map(|details| details.collection_id.clone())
            })
            .collect();
        assert_eq!(all_results.len(), 2);
        assert_eq!(collections.len(), 2);
        assert!(collections.contains("A1"));
        assert!(collections.contains("Default"));

        let response = collection_routes()
            .with_state(state)
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/search?collection=A1&pageSize=24&offset=0")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"name":"War Priest"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let a1_results: Vec<ResultCard> = serde_json::from_slice(&body).unwrap();
        assert_eq!(a1_results.len(), 1);
        assert_eq!(a1_results[0].mtg_card.id, card_id);
        let details = a1_results[0].mtg_card.details.as_ref().unwrap();
        assert_eq!(details.collection_id, "A1");
        assert_eq!(details.quantity, 1);
    }

    #[tokio::test]
    async fn collection_api_renames_and_marks_proxy() {
        let state = test_state();
        {
            let storage = &mut state.1.lock().await.storage;
            storage.add_collection("Binder".to_string()).await.unwrap();
        }

        let response = collection_routes()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/rename/Binder")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"id":"Proxy Binder"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let renamed: Collection = serde_json::from_slice(&body).unwrap();
        assert_eq!(renamed.id, "Proxy Binder");

        let response = collection_routes()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/proxy/Proxy%20Binder")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"isProxy":true}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let proxied: Collection = serde_json::from_slice(&body).unwrap();
        assert_eq!(proxied.id, "Proxy Binder");
        assert!(proxied.is_proxy);
    }

    #[tokio::test]
    async fn import_accepts_pasted_text_without_file() {
        let retrieval = RetrievalSystem::MagicSQLiteRetrievalSystem(
            MagicSQLiteRetrievalSystem::new(None).unwrap(),
        );
        let storage = PersistenceSystem::SQLitePersistenceSystem(
            SQLitePersistenceSystem::new(true, None).unwrap(),
        );
        let state = (
            Arc::new(Mutex::new(crate::RetrievalState {
                mtg: Some(retrieval),
                riftbound: None,
                pokemon: None,
                mtg_system_type: Some(crate::Systems::Sql),
                mtg_db_path: None,
                riftbound_db_path: None,
                pokemon_db_path: None,
                downloading: HashMap::new(),
            })),
            Arc::new(Mutex::new(crate::StorageState {
                storage,
                _storage_db_path: None,
            })),
        );

        let boundary = "gathers-test-boundary";
        let csv = "Name,Set code,Set name,Collector number,Foil,Rarity,Quantity,ManaBox ID,Scryfall ID,Purchase price,Misprint,Altered,Condition,Language,Purchase price currency\r\nSerra Angel,m13,Magic 2013,39,normal,uncommon,2,32634,780f9197-e910-4c7a-bb4b-2c4a94903c39,0.8,false,false,near_mint,en,USD\r\nAvacyn's Pilgrim,isd,Innistrad,173,foil,common,4,32635,00000000-0000-0000-0000-000000000000,0,false,false,near_mint,en,USD\r\n";
        let body = format!(
            "--{boundary}\r\n\
             Content-Disposition: form-data; name=\"collection\"\r\n\r\n\
             Pasted Collection\r\n\
             --{boundary}\r\n\
             Content-Disposition: form-data; name=\"text\"\r\n\r\n\
             {csv}\r\n\
             --{boundary}--\r\n"
        );

        let response = collection_routes()
            .with_state(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/import")
                    .header(
                        "content-type",
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let card_count = state
            .1
            .lock()
            .await
            .storage
            .get_cards_in_collection_count(
                "Pasted Collection".to_string(),
                &[],
                persistence::ProxyFilter::Include,
            )
            .await
            .unwrap();
        assert_eq!(card_count, 2);
    }
}
