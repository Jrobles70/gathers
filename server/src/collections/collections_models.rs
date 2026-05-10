use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use crate::mtg_api::mtg_api_models::{APICardColour, APIRarity};
use crate::pokemon_api::pokemon_api_models::APIEnergyType;
use crate::riftbound_api::riftbound_api_models::APICardDomain;
use persistence;

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, PartialEq)]
pub enum APISortField {
    #[default]
    Name,
    Rarity,
    SetCode,
    CollectorNumber,
    Artist,
}

impl From<APISortField> for models::filters::SortField {
    fn from(value: APISortField) -> Self {
        match value {
            APISortField::Name => models::filters::SortField::Name,
            APISortField::Rarity => models::filters::SortField::Rarity,
            APISortField::SetCode => models::filters::SortField::SetCode,
            APISortField::CollectorNumber => models::filters::SortField::CollectorNumber,
            APISortField::Artist => models::filters::SortField::Artist,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, PartialEq)]
pub enum APISortOrder {
    #[default]
    Asc,
    Desc,
}

impl From<APISortOrder> for models::filters::SortOrder {
    fn from(value: APISortOrder) -> Self {
        match value {
            APISortOrder::Asc => models::filters::SortOrder::Asc,
            APISortOrder::Desc => models::filters::SortOrder::Desc,
        }
    }
}

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
    #[serde(default, deserialize_with = "empty_string_to_none")]
    pub rarity: Option<APIRarity>,
    pub subtypes: Option<Vec<String>>,
    pub supertypes: Option<String>,
    pub types: Option<Vec<String>>,
    pub domains: Option<Vec<APICardDomain>>,
    #[serde(alias = "energyTypes")]
    pub energy_types: Option<Vec<APIEnergyType>>,
    #[serde(alias = "sortBy")]
    pub sort_by: Option<APISortField>,
    #[serde(alias = "sortOrder")]
    pub sort_order: Option<APISortOrder>,
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
            domains: value.domains.map(|v| {
                v.into_iter()
                    .map(models::riftbound::CardDomain::from)
                    .collect()
            }),
            energy_types: value.energy_types.map(|v| {
                v.into_iter()
                    .map(models::pokemon::EnergyType::from)
                    .collect()
            }),
            sort_by: value.sort_by.map(models::filters::SortField::from),
            sort_order: value.sort_order.map(models::filters::SortOrder::from),
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
    #[serde(rename = "purchasePriceCents")]
    pub purchase_price_cents: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema, PartialEq)]
pub enum APICollectionSortField {
    #[default]
    TimeAdded,
    Quantity,
    FoilQuantity,
    Provider,
}

impl From<APICollectionSortField> for persistence::CollectionSortField {
    fn from(value: APICollectionSortField) -> Self {
        match value {
            APICollectionSortField::TimeAdded => persistence::CollectionSortField::TimeAdded,
            APICollectionSortField::Quantity => persistence::CollectionSortField::Quantity,
            APICollectionSortField::FoilQuantity => persistence::CollectionSortField::FoilQuantity,
            APICollectionSortField::Provider => persistence::CollectionSortField::Provider,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct CollectionCardsQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(alias = "pageSize")]
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub sort_by: Option<APICollectionSortField>,
    pub sort_order: Option<APISortOrder>,
    /// Single provider inclusion filter (legacy, takes precedence over `providers`).
    pub provider: Option<String>,
    /// Multiple provider inclusion filter — comma-separated: `?providers=X,Y`.
    #[serde(default)]
    pub providers: Option<String>,
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
    #[serde(default)]
    pub provider: String,
    #[serde(rename = "purchasePrice", skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<PurchasePrice>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
pub struct PurchasePrice {
    #[serde(rename = "usdCents")]
    pub usd_cents: i64,
    pub source: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, JsonSchema)]
pub struct PurchasePriceUpdate {
    pub id: String,
    #[serde(rename = "purchasePriceCents")]
    pub purchase_price_cents: Option<i64>,
}

#[derive(Serialize, JsonSchema)]
pub struct CollectionPriceStats {
    #[serde(rename = "collectionId", skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    #[serde(rename = "cardCount")]
    pub card_count: usize,
    #[serde(rename = "copyCount")]
    pub copy_count: i64,
    #[serde(rename = "pricedCopyCount")]
    pub priced_copy_count: i64,
    #[serde(rename = "baselineCopyCount")]
    pub baseline_copy_count: i64,
    #[serde(rename = "totalValueCents")]
    pub total_value_cents: i64,
    #[serde(rename = "trackedCurrentValueCents")]
    pub tracked_current_value_cents: i64,
    #[serde(rename = "purchaseValueCents")]
    pub purchase_value_cents: i64,
    #[serde(rename = "changeCents", skip_serializing_if = "Option::is_none")]
    pub change_cents: Option<i64>,
    #[serde(rename = "changePercent", skip_serializing_if = "Option::is_none")]
    pub change_percent: Option<f64>,
}

impl From<&CollectionCard> for models::CollectionCard {
    fn from(value: &CollectionCard) -> Self {
        models::CollectionCard {
            uuid: value.id.to_string(),
            quantity: value.quantity,
            foil_quantity: value.foil_quantity,
            collection: value.collection_id.to_string(),
            time_added: value.time_added.to_string(),
            provider: value.provider.clone(),
            purchase_price_cents: value.purchase_price.as_ref().map(|p| p.usd_cents),
            purchase_price_source: value.purchase_price.as_ref().map(|p| p.source.clone()),
            purchase_price_updated_at: value.purchase_price.as_ref().map(|p| p.updated_at.to_rfc3339()),
        }
    }
}

fn default_limit() -> usize {
    24
}

#[derive(Deserialize, JsonSchema)]
pub struct CollectionsSearchQuery {
    #[serde(default)]
    pub offset: usize,
    #[serde(alias = "pageSize")]
    #[serde(default = "default_limit")]
    pub page_size: usize,
    #[serde(default, alias = "skipNotOwned")]
    pub skip_not_owned: bool,
    pub collection: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<crate::mtg_api::mtg_api_models::APICardPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<CollectionCard>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
pub struct ResultCard {
    #[serde(rename = "mtGCard")]
    pub mtg_card: ResultCardInner,
}

fn empty_string_to_none<'de, D>(deserializer: D) -> Result<Option<APIRarity>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(match opt.as_deref() {
        Some("") | None => None,
        Some(s) => {
            Some(serde_json::from_str(&format!("\"{}\"", s)).map_err(serde::de::Error::custom)?)
        }
    })
}
