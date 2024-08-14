use serde_json::Value;

pub struct ScryfallRetrievalSystem {}

use models::filters::CardSearchFilters;

use crate::RetrievalSystemTrait;

impl RetrievalSystemTrait for ScryfallRetrievalSystem {
    async fn get_card(&self, filters: CardSearchFilters) -> eyre::Result<models::Card> {
        let url = format!("https://api.scryfall.com/cards/named?fuzzy={}", filters.card_name.unwrap_or("panharmonicon".to_string())).to_string();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("gathers_cli/1.0"));
        headers.insert("Accept", reqwest::header::HeaderValue::from_static("*/*"));
        let client = reqwest::Client::new();
        let response = client.get(url).headers(headers).send().await?;
        let json: Value = response.json().await?;
        println!("{:?}", json);
        let card_name = json.get("name");
        Ok(models::Card { name: card_name.expect("Ouchies").to_string() })
    }
}
