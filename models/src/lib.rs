use filters::{CardColour, Rarity};
use serde::{Deserialize, Serialize};
pub mod filters;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub set_code: String,
    pub rarity: Rarity,
    pub artist: String,
    pub color_identity: Vec<CardColour>,
    pub text: String,
    pub card_identifiers: CardIdentifiers,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CardIdentifiers {
    pub id: String,
    pub scryfall_id: String,
}

impl Default for Card {
    fn default() -> Card {
        Card {
            id: "".to_string(),
            name: "".to_string(),
            set_code: "".to_string(),
            rarity: Rarity::Common,
            artist: "".to_string(),
            color_identity: vec![],
            text: "".to_string(),
            card_identifiers: CardIdentifiers {
                id: "-1".to_string(),
                scryfall_id: "".to_string(),
            },
        }
    }
}
