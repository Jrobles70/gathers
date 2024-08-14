mod systems;

pub use systems::dummy::DummyRetrievalSystem;
pub use systems::scryfall::ScryfallRetrievalSystem;

pub enum RetrievalSystem {
    Scryfall(ScryfallRetrievalSystem),
    //Database(DatabaseRetrievalSystem),
    Dummy(DummyRetrievalSystem),
}

pub trait RetrievalSystemTrait {
    async fn get_card(&self, filters: models::filters::CardSearchFilters) -> eyre::Result<models::Card>;
}

impl RetrievalSystemTrait for RetrievalSystem {
    async fn get_card(&self, filters: models::filters::CardSearchFilters) -> eyre::Result<models::Card> {
        match self {
            RetrievalSystem::Dummy(d) => d.get_card(filters).await,
            RetrievalSystem::Scryfall(d) => d.get_card(filters).await,
        }
    }
}
