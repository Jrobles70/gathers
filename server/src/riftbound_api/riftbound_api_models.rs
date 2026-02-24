use models::riftbound::{CardDomain, RiftboundCard};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct APIRiftboundCard {
    pub id: String,
    pub name: String,
    #[serde(rename = "setCode")]
    pub set_code: String,
    pub rarity: RBRarity,
    pub artists: Vec<String>,
    pub domains: Vec<CardDomain>,
    pub text: String,
    pub image: String,
}

impl From<RiftboundCard> for APIRiftboundCard {
    fn from(value: RiftboundCard) -> Self {
        APIRiftboundCard {
            id: value.id,
            name: value.name,
            set_code: value.set_code,
            rarity: value.rarity.into(),
            artists: value.artists,
            domains: value.domains,
            text: value.text,
            image: value.image,
        }
    }
}

// Rarity enum for Riftbound cards
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RBRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Showcase,
}

impl std::fmt::Display for RBRarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RBRarity::Common => write!(f, "Common"),
            RBRarity::Uncommon => write!(f, "Uncommon"),
            RBRarity::Rare => write!(f, "Rare"),
            RBRarity::Epic => write!(f, "Epic"),
            RBRarity::Showcase => write!(f, "Showcase"),
        }
    }
}

impl From<models::riftbound::RBRarity> for RBRarity {
    fn from(value: models::riftbound::RBRarity) -> Self {
        match value {
            models::riftbound::RBRarity::Common => RBRarity::Common,
            models::riftbound::RBRarity::Uncommon => RBRarity::Uncommon,
            models::riftbound::RBRarity::Rare => RBRarity::Rare,
            models::riftbound::RBRarity::Epic => RBRarity::Epic,
            models::riftbound::RBRarity::Showcase => RBRarity::Showcase,
        }
    }
}
