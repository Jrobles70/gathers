use std::collections::HashMap;

use eyre::OptionExt;
use models::{
    filters::CardSearchFilters, CardColour, CardID, CardIdentifiers, CollectorNumber, MagicCard,
    SetCode,
};
use serde_json::Value;

use crate::{NamedRetrievalSystem, RetrievalSystemTrait};

#[derive(Debug, Clone)]
pub struct ScryfallRetrievalSystem {}

impl NamedRetrievalSystem for ScryfallRetrievalSystem {}

impl ScryfallRetrievalSystem {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {})
    }
}

impl RetrievalSystemTrait for ScryfallRetrievalSystem {
    #[allow(unused_variables)]
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
        let mut result = HashMap::new();

        for id in ids {
            let url = format!("https://api.scryfall.com/cards/{}", id);
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

            let card = models::MagicCard {
                name: card_name.to_string(),
                set_code: json
                    .get("set")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve set")?
                    .to_string(),
                artist: json
                    .get("artist")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve artist")?
                    .to_string(),
                color_identity: json
                    .get("color_identity")
                    .and_then(Value::as_array)
                    .ok_or_eyre("Could not retrieve color identity")?
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
                    .ok_or_eyre("Could not retrieve rarity")?
                    .to_string()
                    .into(),
                text: json
                    .get("oracle_text")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve oracle text")?
                    .to_string(),
                card_identifiers: CardIdentifiers {
                    scryfall_id: card_id.clone(),
                    id: card_id.clone(),
                },
                collector_number: json
                    .get("collector_number")
                    .and_then(Value::as_str)
                    .ok_or_eyre("Could not retrieve collector number")?
                    .to_string(),
            };

            result.insert(id, card);
        }

        Ok(result)
    }

    async fn get_sets(&self) -> eyre::Result<Vec<models::Set>> {
        // TODO: implement this
        Ok(vec![])
    }

    #[allow(unused_variables)]
    async fn bulk_search_cards(
        &self,
        cards: Vec<(SetCode, CollectorNumber)>,
    ) -> eyre::Result<Vec<(SetCode, CollectorNumber, CardID)>> {
        Ok(vec![])
    }

    async fn update_backend(&self) -> eyre::Result<bool> {
        Ok(false)
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

    #[tokio::test]
    async fn get_cards_by_ids() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem {};
        let test_ids = vec![
            "998d0cc8-ca2a-41c3-ab65-d05c26ab8278".to_string(),
            "9a6cd6f6-ae6e-4a77-95ca-64c6882357d5".to_string(),
            "70dd138f-391a-4956-bc2a-fe186429c71a".to_string(),
        ];

        let result = r.get_cards_by_ids(test_ids).await?;

        assert_eq!(result.len(), 3);
        assert!(result.contains_key("998d0cc8-ca2a-41c3-ab65-d05c26ab8278"));
        assert!(result.contains_key("9a6cd6f6-ae6e-4a77-95ca-64c6882357d5"));
        assert!(result.contains_key("70dd138f-391a-4956-bc2a-fe186429c71a"));

        Ok(())
    }
}
