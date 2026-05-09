use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{Artist, CardID, CardName, CardText, CardTrait, CollectorNumber, SetCode};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct MagicCard {
    pub id: CardID,
    pub name: CardName,
    pub set_code: SetCode,
    pub collector_number: CollectorNumber,
    pub rarity: Rarity,
    pub artist: Artist,
    pub color_identity: Vec<CardColour>,
    pub text: CardText,
    pub card_identifiers: CardIdentifiers,
    pub subtypes: Vec<String>,
    pub supertypes: Vec<String>,
    pub types: Vec<String>,
}

impl CardTrait for MagicCard {
    fn get_set(&self) -> SetCode {
        self.set_code.clone()
    }

    fn get_collector_number(&self) -> CollectorNumber {
        self.collector_number.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Mythic,
    Special,
    Bonus,
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
    #[serde(alias = "Colorless")]
    Colourless,
    Multicoloured,
}

impl Default for MagicCard {
    fn default() -> MagicCard {
        MagicCard {
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
            subtypes: vec![],
            supertypes: vec![],
            types: vec![],
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

impl From<&String> for CardColour {
    fn from(value: &String) -> Self {
        match value.to_lowercase().as_str() {
            "w" | "white" => CardColour::White,
            "u" | "blue" => CardColour::Blue,
            "b" | "black" => CardColour::Black,
            "r" | "red" => CardColour::Red,
            "g" | "green" => CardColour::Green,
            "c" | "colourless" | "colorless" => CardColour::Colourless,
            "m" | "multicoloured" => CardColour::Multicoloured,
            _ => CardColour::Colourless,
        }
    }
}

impl std::fmt::Display for Rarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rarity::Common => write!(f, "Common"),
            Rarity::Uncommon => write!(f, "Uncommon"),
            Rarity::Rare => write!(f, "Rare"),
            Rarity::Mythic => write!(f, "Mythic"),
            Rarity::Special => write!(f, "Special"),
            Rarity::Bonus => write!(f, "Bonus"),
        }
    }
}

impl Rarity {
    pub fn to_single_string(&self) -> String {
        match self {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Mythic => "mythic",
            Rarity::Special => "special",
            Rarity::Bonus => "bonus",
        }
        .to_string()
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
            _ => Rarity::Bonus,
        }
    }
}
