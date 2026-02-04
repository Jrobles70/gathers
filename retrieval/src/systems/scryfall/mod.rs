use std::collections::HashMap;

use eyre::OptionExt;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScryfallRetrievalSystem {}

use models::{
    filters::CardSearchFilters, CardColour, CardID, CardIdentifiers, CollectorNumber, MagicCard,
    SetCode,
};

use crate::RetrievalSystemTrait;

impl ScryfallRetrievalSystem {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {})
    }
}

impl RetrievalSystemTrait for ScryfallRetrievalSystem {
    async fn search_cards(
        &self,
        filters: CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<MagicCard>> {
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
        println!("{json:?}");
        let card_name = json
            .get("name")
            .and_then(Value::as_str)
            .ok_or_eyre("Could not retrieve name")?;
        let card_id = json
            .get("id")
            .and_then(Value::as_str)
            .ok_or_eyre("Could not retrieve id")?
            .to_string();
        Ok(vec![models::MagicCard {
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
                    "U" => CardColour::Blue,
                    "W" => CardColour::White,
                    "G" => CardColour::Green,
                    "R" => CardColour::Red,
                    "C" => CardColour::Colourless,
                    _ => CardColour::Colourless,
                })
                .collect::<Vec<CardColour>>(),
            id: card_id.clone(),
            rarity: json
                .get("rarity")
                .and_then(Value::as_str)
                .ok_or_eyre("Oh no")?
                .to_string()
                .into(),
            text: json
                .get("oracle_text")
                .and_then(Value::as_str)
                .ok_or_eyre("Oh no")?
                .to_string(),
            card_identifiers: CardIdentifiers {
                scryfall_id: card_id.clone(),
                id: card_id.clone(),
            },
            collector_number: json
                .get("collector_number")
                .and_then(Value::as_str)
                .ok_or_eyre("Oh no")?
                .to_string(),
        }])
    }

    async fn get_cards_by_ids(
        &self,
        ids: Vec<String>,
    ) -> eyre::Result<HashMap<String, models::MagicCard>> {
        // TODO: implement this
        Ok(HashMap::new())
    }

    async fn get_sets(&self) -> eyre::Result<Vec<models::Set>> {
        // TODO: implement this
        Ok(vec![])
    }

    async fn bulk_search_cards(
        &self,
        cards: Vec<(SetCode, CollectorNumber)>,
    ) -> eyre::Result<Vec<(SetCode, CollectorNumber, CardID)>> {
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
