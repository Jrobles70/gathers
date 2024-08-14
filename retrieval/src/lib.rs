mod systems;

pub use systems::dummy::DummyRetrievalSystem;

pub enum RetrievalSystem {
    //Scryfall(ScryfallRetrievalSystem),
    //Database(DatabaseRetrievalSystem),
    Dummy(DummyRetrievalSystem),
}

pub trait RetrievalSystemTrait {
    fn get_card(&self) -> models::Card;
}

impl RetrievalSystemTrait for RetrievalSystem {
    fn get_card(&self) -> models::Card {
        match self {
            RetrievalSystem::Dummy(d) => d.get_card(),
        }
    }
}
