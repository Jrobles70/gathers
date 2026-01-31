use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
}

#[derive(Serialize)]
pub struct CollectionAddResponse {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct CollectionRemoveResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct MoveCardsResponse {
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct CardToAdd {
    pub id: String,
    pub quantity: i32,
    #[serde(rename = "foilQuantity")]
    pub foil_quantity: i32,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
}

#[derive(Deserialize)]
pub struct CollectionCardsQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Serialize, Deserialize)]
pub struct CollectionCard {
    pub id: String,
    pub quantity: u32,
    #[serde(rename = "foilQuantity")]
    pub foil_quantity: u32,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
    #[serde(rename = "timeAdded")]
    pub time_added: DateTime<Utc>,
}

impl From<&CollectionCard> for models::CollectionCard {
    fn from(value: &CollectionCard) -> Self {
        models::CollectionCard {
            uuid: value.id.to_string(),
            quantity: value.quantity,
            foil_quantity: value.foil_quantity,
            collection: value.collection_id.to_string(),
            time_added: value.time_added.to_string(),
        }
    }
}

fn default_limit() -> usize {
    20
}

#[derive(Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub page_size: usize,
}

#[derive(Deserialize, Serialize)]
pub struct CardIdentInner {
    #[serde(rename = "scryfallId")]
    pub scryfall_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct ResultCardInner {
    pub id: String,
    pub name: String,
    #[serde(rename = "setCode")]
    pub set_code: String,
    #[serde(rename = "cardIdentifiers")]
    pub card_identifiers: CardIdentInner,
}

#[derive(Deserialize, Serialize)]
pub struct ResultCard {
    #[serde(rename = "mtGCard")]
    pub mtg_card: ResultCardInner,
}
