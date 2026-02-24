use serde::{Deserialize, Serialize};

use crate::{Artist, CardID, CardName, CardText, CardTrait, CollectorNumber, SetCode};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct RiftboundCard {
    pub id: CardID,
    pub name: CardName,
    pub set_code: SetCode,
    pub collector_number: CollectorNumber,
    pub rarity: RBRarity,
    pub artists: Artist,
    pub domains: Vec<CardDomain>,
    pub text: CardText,
    pub image: String,
}

impl CardTrait for RiftboundCard {
    fn get_set(&self) -> SetCode {
        self.set_code.clone()
    }

    fn get_collector_number(&self) -> CollectorNumber {
        self.collector_number.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum RBRarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Showcase,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CardDomain {
    Calm,
    Chaos,
    Fury,
    Mind,
    Body,
    Order,
    Colorless,
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

impl RBRarity {
    pub fn to_single_string(&self) -> String {
        match self {
            RBRarity::Common => "common",
            RBRarity::Uncommon => "uncommon",
            RBRarity::Rare => "rare",
            RBRarity::Epic => "epic",
            RBRarity::Showcase => "showcase",
        }
        .to_string()
    }
}

impl From<String> for RBRarity {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Common" | "common" | "c" => RBRarity::Common,
            "Uncommon" | "uncommon" | "u" => RBRarity::Uncommon,
            "Rare" | "rare" | "r" => RBRarity::Rare,
            "Epic" | "epic" | "e" => RBRarity::Epic,
            "Showcase" | "showcase" => RBRarity::Showcase,
            _ => RBRarity::Showcase,
        }
    }
}
