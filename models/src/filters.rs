use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSearchFilters {
    pub name: Option<String>,
    #[serde(alias = "colorIdentities")]
    pub color_identities: Option<Vec<CardColour>>,
    #[serde(alias = "setCode")]
    pub set_code: Option<String>,
    #[serde(alias = "collectorNumber")]
    pub collector_number: Option<String>,
    pub artist: Option<String>,
    pub text: Option<String>,
    pub rarity: Option<Rarity>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Mythic,
    // TODO if others
}

impl Default for CardSearchFilters {
    fn default() -> CardSearchFilters {
        CardSearchFilters {
            name: None,
            color_identities: None,
            set_code: None,
            collector_number: None,
            artist: None,
            text: None,
            rarity: None,
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
            _ => Rarity::Common,
        }
    }
}
