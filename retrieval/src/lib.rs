mod systems;

use std::collections::HashMap;

use models::{CardID, CollectorNumber, SetCode};
pub use systems::scryfall::ScryfallRetrievalSystem;
pub use systems::sqlite::MagicSQLiteRetrievalSystem;

#[derive(Debug, Clone)]
pub enum RetrievalSystem {
    Scryfall(ScryfallRetrievalSystem),
    Database(MagicSQLiteRetrievalSystem),
}

#[async_trait::async_trait]
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

#[async_trait::async_trait]
impl RetrievalSystemTrait for RetrievalSystem {
    async fn search_cards(
        &self,
        filters: models::filters::CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<models::MagicCard>> {
        match self {
            RetrievalSystem::Scryfall(d) => d.search_cards(filters, skip, limit).await,
            RetrievalSystem::Database(d) => d.search_cards(filters, skip, limit).await,
        }
    }

    async fn get_cards_by_ids(
        &self,
        ids: Vec<String>,
    ) -> eyre::Result<HashMap<String, models::MagicCard>> {
        match self {
            RetrievalSystem::Scryfall(d) => d.get_cards_by_ids(ids).await,
            RetrievalSystem::Database(d) => d.get_cards_by_ids(ids).await,
        }
    }

    async fn get_sets(&self) -> eyre::Result<Vec<models::Set>> {
        match self {
            RetrievalSystem::Scryfall(d) => d.get_sets().await,
            RetrievalSystem::Database(d) => d.get_sets().await,
        }
    }

    async fn bulk_search_cards(
        &self,
        cards: Vec<(SetCode, CollectorNumber)>,
    ) -> eyre::Result<Vec<(SetCode, CollectorNumber, CardID)>> {
        match self {
            RetrievalSystem::Scryfall(d) => d.bulk_search_cards(cards).await,
            RetrievalSystem::Database(d) => d.bulk_search_cards(cards).await,
        }
    }
}
