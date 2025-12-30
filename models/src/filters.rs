use serde::{Deserialize, Deserializer, Serialize};

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
    #[serde(default, deserialize_with = "empty_string_to_none")]
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

fn empty_string_to_none<'de, D>(deserializer: D) -> Result<Option<Rarity>, D::Error>
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
