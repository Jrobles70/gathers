use models::{CardColour, CardIdentifiers, MagicCard, Rarity};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct APICard {
    pub id: String,
    pub name: String,
    #[serde(rename = "setCode")]
    pub set_code: String,
    pub rarity: Rarity,
    pub artist: String,
    #[serde(rename = "colorIdentity")]
    pub color_identity: Vec<CardColour>,
    pub text: String,
    #[serde(rename = "cardIdentifiers")]
    pub card_identifiers: APICardIdentifiers,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct APICardIdentifiers {
    pub id: String,
    #[serde(rename = "scryfallId")]
    pub scryfall_id: String,
}

impl From<MagicCard> for APICard {
    fn from(value: MagicCard) -> Self {
        APICard {
            id: value.id,
            name: value.name,
            set_code: value.set_code,
            rarity: value.rarity,
            artist: value.artist,
            color_identity: value.color_identity,
            text: value.text,
            card_identifiers: value.card_identifiers.into(),
        }
    }
}

impl From<CardIdentifiers> for APICardIdentifiers {
    fn from(value: CardIdentifiers) -> Self {
        APICardIdentifiers {
            id: value.id,
            scryfall_id: value.scryfall_id,
        }
    }
}
