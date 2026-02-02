use std::fmt::Display;

use serde::{Deserialize, Serialize};
pub mod filters;

pub type Artist = String;
pub type CardID = String;
pub type SetCode = String;
pub type CardText = String;
pub type CardName = String;
pub type SetName = String;
pub type CollectionID = String;
pub type CollectorNumber = String;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Card {
    pub id: CardID,
    pub name: CardName,
    pub set_code: SetCode,
    pub collector_number: CollectorNumber,
    pub rarity: Rarity,
    pub artist: Artist,
    pub color_identity: Vec<CardColour>,
    pub text: CardText,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Set {
    pub name: SetName,
    pub code: SetCode,
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
            collector_number: "".to_string(),
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
            "Common" | "common" | "c" => Rarity::Common,
            "Uncommon" | "uncommon" | "u" => Rarity::Uncommon,
            "Rare" | "rare" | "r" => Rarity::Rare,
            "Mythic" | "mythic" | "m" => Rarity::Mythic,
            "Special" => Rarity::Special,
            _ => Rarity::Common,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionCard {
    pub uuid: CardID,
    pub quantity: u32,
    pub foil_quantity: u32,
    pub time_added: String,
    pub collection: CollectionID,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: CollectionID,
    pub can_remove: bool,
}
