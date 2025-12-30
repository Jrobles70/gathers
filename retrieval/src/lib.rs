mod systems;

use std::collections::HashMap;

// pub use systems::dummy::DummyRetrievalSystem;
// pub use systems::scryfall::ScryfallRetrievalSystem;
pub use systems::sqlite::SQLiteRetrievalSystem;

#[derive(Debug, Clone)]
pub enum RetrievalSystem {
    // Scryfall(ScryfallRetrievalSystem),
    Database(SQLiteRetrievalSystem),
    // Dummy(DummyRetrievalSystem),
}

#[async_trait::async_trait]
pub trait RetrievalSystemTrait {
    async fn search_cards(
        &self,
        filters: models::filters::CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<models::Card>>;

    async fn get_cards_by_ids(
        &self,
        ids: Vec<String>,
    ) -> eyre::Result<HashMap<String, models::Card>>;
}

#[async_trait::async_trait]
impl RetrievalSystemTrait for RetrievalSystem {
    async fn search_cards(
        &self,
        filters: models::filters::CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<models::Card>> {
        match self {
            // RetrievalSystem::Dummy(d) => d.get_card(filters).await,
            // RetrievalSystem::Scryfall(d) => d.get_card(filters).await,
            RetrievalSystem::Database(d) => d.search_cards(filters, skip, limit).await,
            _ => todo!(),
        }
    }

    async fn get_cards_by_ids(
        &self,
        ids: Vec<String>,
    ) -> eyre::Result<HashMap<String, models::Card>> {
        match self {
            // RetrievalSystem::Dummy(_) => todo!(),
            // RetrievalSystem::Scryfall(_) => todo!(),
            RetrievalSystem::Database(d) => d.get_cards_by_ids(ids).await,
        }
    }
}
