use serde::{Deserialize, Deserializer, Serialize};

use crate::{CardColour, Rarity, pokemon::EnergyType, riftbound::CardDomain};

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
    pub subtypes: Option<Vec<String>>,
    pub supertypes: Option<String>,
    pub types: Option<Vec<String>>,
    /// Riftbound-only: filter by one or more card domains.
    pub domains: Option<Vec<CardDomain>>,
    /// Pokemon-only: filter by one or more energy types.
    #[serde(alias = "energyTypes")]
    pub energy_types: Option<Vec<EnergyType>>,
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
            subtypes: None,
            supertypes: None,
            types: None,
            domains: None,
            energy_types: None,
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

    pub fn with_set_code(mut self, set_code: impl Into<String>) -> Self {
        self.set_code = Some(set_code.into());
        self
    }

    pub fn with_collector_number(mut self, collector_number: impl Into<String>) -> Self {
        self.collector_number = Some(collector_number.into());
        self
    }

    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = Some(artist.into());
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn with_rarity(mut self, rarity: Rarity) -> Self {
        self.rarity = Some(rarity);
        self
    }

    pub fn with_subtypes(mut self, subtypes: Vec<String>) -> Self {
        self.subtypes = Some(subtypes);
        self
    }

    pub fn with_supertypes(mut self, supertypes: impl Into<String>) -> Self {
        self.supertypes = Some(supertypes.into());
        self
    }

    pub fn with_types(mut self, types: Vec<String>) -> Self {
        self.types = Some(types);
        self
    }

    pub fn with_domains(mut self, domains: Vec<CardDomain>) -> Self {
        self.domains = Some(domains);
        self
    }

    pub fn with_energy_types(mut self, energy_types: Vec<EnergyType>) -> Self {
        self.energy_types = Some(energy_types);
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
