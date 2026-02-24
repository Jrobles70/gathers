use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
pub mod filters;
pub mod mtg;
pub mod riftbound;

pub use mtg::{CardColour, CardIdentifiers, MagicCard, Rarity};

use crate::riftbound::RiftboundCard;

pub type Artist = String;
pub type CardID = String;
pub type SetCode = String;
pub type CardText = String;
pub type CardName = String;
pub type SetName = String;
pub type CollectionID = String;
pub type CollectorNumber = String;

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum Card {
    Magic(MagicCard),
    Riftbound(RiftboundCard),
}

#[enum_dispatch(Card)]
pub trait CardTrait {
    fn get_set(&self) -> SetCode;
    fn get_collector_number(&self) -> CollectorNumber;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Set {
    pub name: SetName,
    pub code: SetCode,
}

#[derive(Debug, Clone)]
pub struct CollectionCard {
    pub uuid: CardID,
    pub quantity: i32,
    pub foil_quantity: i32,
    pub time_added: String,
    pub collection: CollectionID,
    pub provider: String,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: CollectionID,
    pub can_remove: bool,
}
