use models::pokemon::{EnergyType, PokemonCard, PokemonRarity};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct APIPokemonCard {
    pub id: String,
    pub name: String,
    #[serde(rename = "setCode")]
    pub set_code: String,
    pub rarity: PokemonRarity,
    pub image: String,
    #[serde(rename = "energyTypes")]
    pub energy_types: Vec<EnergyType>,
    #[serde(rename = "cardType")]
    pub card_type: String,
    #[serde(rename = "collectorNumber")]
    pub collector_number: String,
}

impl From<PokemonCard> for APIPokemonCard {
    fn from(value: PokemonCard) -> Self {
        APIPokemonCard {
            id: value.id,
            name: value.name,
            set_code: value.set_code,
            rarity: value.rarity,
            image: value.image,
            energy_types: value.energy_types,
            card_type: value.card_type,
            collector_number: value.collector_number,
        }
    }
}
