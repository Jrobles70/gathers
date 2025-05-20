#[derive(Debug, Clone)]
pub struct DummyRetrievalSystem {}

use models::filters::CardSearchFilters;

use crate::RetrievalSystemTrait;

#[async_trait::async_trait]
impl RetrievalSystemTrait for DummyRetrievalSystem {
    async fn get_card(&self, filters: CardSearchFilters) -> eyre::Result<Option<models::Card>> {
        println!("Got request.");

        Ok(Some(models::Card {
            name: filters.name.unwrap_or("Other".to_string()),
            set_code: "kld".to_string(),
            ..Default::default()
        }))
    }
}
