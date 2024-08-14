pub struct DummyRetrievalSystem { }

use models::filters::CardSearchFilters;

use crate::RetrievalSystemTrait;

impl RetrievalSystemTrait for DummyRetrievalSystem {
    async fn get_card(&self, filters: CardSearchFilters) -> eyre::Result<models::Card> {
        println!("Got request.");

        Ok(models::Card { name: filters.card_name.unwrap_or("Other".to_string()) })
    }
}
