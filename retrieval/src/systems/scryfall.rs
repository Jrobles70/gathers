use eyre::OptionExt;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ScryfallRetrievalSystem {}

use models::{
    filters::{CardColour, CardSearchFilters},
    CardIdentifiers,
};

use crate::RetrievalSystemTrait;

#[async_trait::async_trait]
impl RetrievalSystemTrait for ScryfallRetrievalSystem {
    async fn get_card(&self, filters: CardSearchFilters) -> eyre::Result<Option<models::Card>> {
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
        println!("{:?}", json);
        let card_name = json
            .get("name")
            .and_then(Value::as_str)
            .ok_or_eyre("Could not retrieve name")?;
        Ok(Some(models::Card {
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
            rarity: models::filters::Rarity::Common,
            text: "a".to_string(),
            card_identifiers: CardIdentifiers {
                scryfall_id: "a".to_string(),
                id: "a".to_string(),
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_basic_card() -> eyre::Result<()> {
        let r = ScryfallRetrievalSystem {};
        let card = r
            .get_card(CardSearchFilters {
                name: Some("Panharmonicon".to_string()),
                ..Default::default()
            })
            .await?;

        assert!(card.is_some());
        let card = card.ok_or_eyre("Card should be present")?;
        assert_eq!(card.name, "Panharmonicon");
        assert_eq!(card.color_identity, vec![]);

        Ok(())
    }
}
