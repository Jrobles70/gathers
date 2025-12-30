use serde::{Deserialize, Serialize};

use crate::{CardColour, Rarity};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl CardSearchFilters {
    pub fn new() -> Self {
        Self {
            name: None,
            color_identities: None,
            set_code: None,
            collector_number: None,
            artist: None,
            text: None,
            rarity: None,
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_color_identities(mut self, identities: Vec<CardColour>) -> Self {
        self.color_identities = Some(identities);
        self
    }
}
