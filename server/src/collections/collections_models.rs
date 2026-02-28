use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::mtg_api::mtg_api_models::{APICardColour, APIRarity};
use crate::pokemon_api::pokemon_api_models::APIEnergyType;
use crate::riftbound_api::riftbound_api_models::APICardDomain;

/// Server-local version of `models::filters::CardSearchFilters` with `JsonSchema`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
pub struct APICardSearchFilters {
    pub name: Option<String>,
    #[serde(alias = "colorIdentities")]
    pub color_identities: Option<Vec<APICardColour>>,
    #[serde(alias = "setCode")]
    pub set_code: Option<String>,
    #[serde(alias = "collectorNumber")]
    pub collector_number: Option<String>,
    pub artist: Option<String>,
    pub text: Option<String>,
    pub rarity: Option<APIRarity>,
    pub subtypes: Option<Vec<String>>,
    pub supertypes: Option<String>,
    pub types: Option<Vec<String>>,
    pub domains: Option<Vec<APICardDomain>>,
    #[serde(alias = "energyTypes")]
    pub energy_types: Option<Vec<APIEnergyType>>,
}

impl From<APICardSearchFilters> for models::filters::CardSearchFilters {
    fn from(value: APICardSearchFilters) -> Self {
        models::filters::CardSearchFilters {
            name: value.name,
            color_identities: value
                .color_identities
                .map(|v| v.into_iter().map(models::CardColour::from).collect()),
            set_code: value.set_code,
            collector_number: value.collector_number,
            artist: value.artist,
            text: value.text,
            rarity: value.rarity.map(models::Rarity::from),
            subtypes: value.subtypes,
            supertypes: value.supertypes,
            types: value.types,
            domains: value
                .domains
                .map(|v| v.into_iter().map(models::riftbound::CardDomain::from).collect()),
            energy_types: value
                .energy_types
                .map(|v| v.into_iter().map(models::pokemon::EnergyType::from).collect()),
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Collection {
    pub id: String,
}

#[derive(Serialize, JsonSchema)]
pub struct CollectionAddResponse {
    pub id: String,
    pub name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct CollectionRemoveResponse {
    pub message: String,
}

#[derive(Deserialize, Debug, JsonSchema)]
pub struct CardToAdd {
    pub id: String,
    pub quantity: i32,
    #[serde(rename = "foilQuantity")]
    pub foil_quantity: i32,
}

#[derive(Deserialize, JsonSchema)]
pub struct CollectionCardsQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct CollectionCard {
    pub id: String,
    pub quantity: i32,
    #[serde(rename = "foilQuantity")]
    pub foil_quantity: i32,
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
            provider: "".to_string(),
        }
    }
}

fn default_limit() -> usize {
    24
}

#[derive(Deserialize, JsonSchema)]
pub struct SearchQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(default = "default_limit")]
    pub page_size: usize,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct CardIdentInner {
    #[serde(rename = "scryfallId")]
    pub scryfall_id: String,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ResultCardInner {
    pub id: String,
    pub name: String,
    #[serde(rename = "setCode")]
    pub set_code: String,
    #[serde(rename = "cardIdentifiers")]
    pub card_identifiers: CardIdentInner,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ResultCard {
    #[serde(rename = "mtGCard")]
    pub mtg_card: ResultCardInner,
}
