use std::collections::HashMap;

use eyre::OptionExt;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScryfallRetrievalSystem {}

use models::{filters::CardSearchFilters, Card, CardColour, CardIdentifiers, Rarity};

use crate::RetrievalSystemTrait;

impl ScryfallRetrievalSystem {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {})
    }
}

#[async_trait::async_trait]
impl RetrievalSystemTrait for ScryfallRetrievalSystem {
    async fn search_cards(
        &self,
        filters: CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<Card>> {
        let url = format!(
            "https://api.scryfall.com/cards/named?fuzzy={}",
            filters.name.unwrap_or("panharmonicon".to_string())
        )
        .to_string();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("gathers_cli/1.0"),
        );
        headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
        let client = reqwest::Client::new();
        let response = client.get(url).headers(headers).send().await?;
        let json: Value = response.json().await?;
        let card_name = json
            .get("name")
            .and_then(Value::as_str)
            .ok_or_eyre("Could not retrieve name")?;
        let card_id = json
            .get("id")
            .and_then(Value::as_str)
            .ok_or_eyre("Could not retrieve id")?
            .to_string();
        Ok(vec![models::Card {
            name: card_name.to_string(),
            set_code: json
                .get("set")
                .and_then(Value::as_str)
                .ok_or_eyre("Could not retrieve printings")?
                .to_string(),
            artist: json
                .get("artist")
                .and_then(Value::as_str)
                .ok_or_eyre("Could not retrieve artist")?
                .to_string(),
            // card_identifiers: json
            //     .get("card_identifiers")
            //     .ok_or_eyre("Could not retrive card identifiers")?,
            color_identity: json
                .get("color_identity")
                .and_then(Value::as_array)
                .ok_or_eyre("Oh no")?
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<&str>>()
                .iter()
                .map(|c| match *c {
                    "B" => CardColour::Black,
                    _ => CardColour::Colourless,
                })
                .collect::<Vec<CardColour>>(),
            id: "a".to_string(),
            rarity: Rarity::Common,
            text: "a".to_string(),
            card_identifiers: CardIdentifiers {
                scryfall_id: card_id.clone(),
                id: card_id.clone(),
            },
        }])
    }

    async fn get_cards_by_ids(
        &self,
        ids: Vec<String>,
    ) -> eyre::Result<HashMap<String, models::Card>> {
        // TODO: implement this
        Ok(HashMap::new())
    }

    async fn get_sets(&self) -> eyre::Result<Vec<models::Set>> {
        // TODO: implement this
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_basic_card() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem {};
        let card = r
            .search_cards(
                CardSearchFilters {
                    name: Some("Panharmonicon".to_string()),
                    ..Default::default()
                },
                None,
                None,
            )
            .await?;

        assert_eq!(card.len(), 1);
        let card = card.first().expect("No card?");
        assert_eq!(card.name, "Panharmonicon");
        assert_eq!(card.color_identity, vec![]);

        Ok(())
    }
}
