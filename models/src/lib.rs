use std::fmt::Display;

use serde::{Deserialize, Serialize};
pub mod filters;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Mythic,
    Special,
    // TODO if others
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct CardIdentifiers {
    pub id: String,
    pub scryfall_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CardColour {
    White,
    Blue,
    Black,
    Red,
    Green,
    Colourless,
    Multicoloured,
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

impl Display for CardColour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CardColour::Red => write!(f, "R"),
            CardColour::White => write!(f, "W"),
            CardColour::Blue => write!(f, "U"),
            CardColour::Green => write!(f, "G"),
            CardColour::Black => write!(f, "B"),
            CardColour::Multicoloured => write!(f, "_"),
            CardColour::Colourless => write!(f, "C"),
        }
    }
}

impl From<String> for Rarity {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Common" => Rarity::Common,
            "Uncommon" => Rarity::Uncommon,
            "Rare" => Rarity::Rare,
            "Mythic" => Rarity::Mythic,
            "Special" => Rarity::Special,
            _ => Rarity::Common,
        }
    }
}
