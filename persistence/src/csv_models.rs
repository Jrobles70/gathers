use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CSVCard {
    #[serde(rename = "Set")]
    pub set_code: String,
    #[serde(rename = "CollectorNumber")]
    pub collector_number: String,
    #[serde(rename = "Quantity")]
    pub quantity: u32,
    #[serde(rename = "FoilQuantity")]
    pub foil_quantity: u32,
}
