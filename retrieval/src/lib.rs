mod systems;

use std::any::type_name;
use std::collections::HashMap;

use enum_dispatch::enum_dispatch;
use models::{CardID, CollectorNumber, SetCode};
pub use systems::scryfall::ScryfallRetrievalSystem;
pub use systems::sqlite::MagicSQLiteRetrievalSystem;

#[enum_dispatch]
#[derive(Debug, Clone)]
pub enum RetrievalSystem {
    ScryfallRetrievalSystem,
    MagicSQLiteRetrievalSystem,
}

#[enum_dispatch(RetrievalSystem)]
pub trait RetrievalSystemTrait {
    async fn search_cards(
        &self,
        filters: models::filters::CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<models::MagicCard>>;

    async fn get_cards_by_ids(
        &self,
        ids: Vec<String>,
    ) -> eyre::Result<HashMap<String, models::MagicCard>>;

    async fn get_sets(&self) -> eyre::Result<Vec<models::Set>>;
    async fn bulk_search_cards(
        &self,
        cards: Vec<(SetCode, CollectorNumber)>,
    ) -> eyre::Result<Vec<(SetCode, CollectorNumber, CardID)>>;
}

pub trait NamedRetrievalSystem {
    fn name(&self) -> &str {
        type_name::<Self>()
    }
}

impl NamedRetrievalSystem for RetrievalSystem {}
