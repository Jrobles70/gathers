mod systems;

pub use systems::dummy::DummyRetrievalSystem;
pub use systems::scryfall::ScryfallRetrievalSystem;
pub use systems::sqlite::SQLiteRetrievalSystem;

pub enum RetrievalSystem {
    Scryfall(ScryfallRetrievalSystem),
    Database(SQLiteRetrievalSystem),
    Dummy(DummyRetrievalSystem),
}

#[async_trait::async_trait]
pub trait RetrievalSystemTrait {
    async fn get_card(&self, filters: models::filters::CardSearchFilters) -> eyre::Result<models::Card>;
}

#[async_trait::async_trait]
impl RetrievalSystemTrait for RetrievalSystem {
    async fn get_card(&self, filters: models::filters::CardSearchFilters) -> eyre::Result<models::Card> {
        match self {
            RetrievalSystem::Dummy(d) => d.get_card(filters).await,
            RetrievalSystem::Scryfall(d) => d.get_card(filters).await,
            RetrievalSystem::Database(d) => d.get_card(filters).await,
        }
    }
}

