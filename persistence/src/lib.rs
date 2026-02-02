mod sqlite;
use std::io;

use models::CardID;
use models::CollectionCard;
use models::CollectionID;
use retrieval::RetrievalSystem;
use retrieval::RetrievalSystemTrait;

pub use crate::sqlite::SQLitePersistenceSystem;

mod csv_models;

#[derive(Debug, Clone)]
pub enum PersistenceSystem {
    Database(SQLitePersistenceSystem),
}

#[async_trait::async_trait]
pub trait PersistenceSystemTrait {
    async fn add_collection(&mut self, name: CollectionID) -> eyre::Result<String>;

    async fn remove_collection(
        &mut self,
        name: CollectionID,
        move_to: Option<CollectionID>,
    ) -> eyre::Result<CollectionID>;

    async fn list_collections(&self) -> eyre::Result<Vec<CollectionID>>;

    async fn get_cards_in_collection_count(
        &self,
        collection_id: CollectionID,
    ) -> eyre::Result<usize>;

    async fn add_card_to_collection(
        &mut self,
        collection_id: CollectionID,
        card_uuid: CardID,
        quantity: i32,
        foil_quantity: i32,
        time_added: String,
    ) -> eyre::Result<CollectionCard>;

    async fn get_cards_in_collection_paginated(
        &self,
        collection_id: CollectionID,
        offset: usize,
        limit: usize,
    ) -> eyre::Result<Vec<CollectionCard>>;

    async fn move_cards_between_collections(
        &mut self,
        cards: Vec<CollectionCard>,
        to_collection_id: CollectionID,
    ) -> eyre::Result<()>;
}

#[async_trait::async_trait]
impl PersistenceSystemTrait for PersistenceSystem {
    async fn add_collection(&mut self, name: CollectionID) -> eyre::Result<CollectionID> {
        match self {
            PersistenceSystem::Database(d) => d.add_collection(name).await,
        }
    }

    async fn remove_collection(
        &mut self,
        name: CollectionID,
        move_to: Option<CollectionID>,
    ) -> eyre::Result<CollectionID> {
        match self {
            PersistenceSystem::Database(d) => d.remove_collection(name, move_to).await,
        }
    }

    async fn list_collections(&self) -> eyre::Result<Vec<CollectionID>> {
        match self {
            PersistenceSystem::Database(d) => d.list_collections().await,
        }
    }

    async fn get_cards_in_collection_count(&self, collection_id: String) -> eyre::Result<usize> {
        match self {
            PersistenceSystem::Database(d) => d.get_cards_in_collection_count(collection_id).await,
        }
    }

    async fn get_cards_in_collection_paginated(
        &self,
        collection_id: CollectionID,
        offset: usize,
        limit: usize,
    ) -> eyre::Result<Vec<CollectionCard>> {
        match self {
            PersistenceSystem::Database(d) => {
                d.get_cards_in_collection_paginated(collection_id, offset, limit)
                    .await
            }
        }
    }

    async fn add_card_to_collection(
        &mut self,
        collection_id: CollectionID,
        card_uuid: CardID,
        quantity: i32,
        foil_quantity: i32,
        time_added: String,
    ) -> eyre::Result<CollectionCard> {
        match self {
            PersistenceSystem::Database(d) => {
                d.add_card_to_collection(
                    collection_id,
                    card_uuid,
                    quantity,
                    foil_quantity,
                    time_added,
                )
                .await
            }
        }
    }

    async fn move_cards_between_collections(
        &mut self,
        cards: Vec<CollectionCard>,
        to_collection_id: CollectionID,
    ) -> eyre::Result<()> {
        match self {
            PersistenceSystem::Database(d) => {
                d.move_cards_between_collections(cards, to_collection_id)
                    .await
            }
        }
    }
}

impl PersistenceSystem {
    pub async fn import_csv(
        &self,
        content: String,
        retrieval: RetrievalSystem,
        progress_sender: Option<tokio::sync::watch::Sender<f32>>,
    ) -> eyre::Result<()> {
        // Get all cards
        let mut rdr = csv::Reader::from_path("/home/mihail/repos/gathers/test.csv")?;
        let mut cards = vec![];
        for result in rdr.deserialize() {
            // Notice that we need to provide a type hint for automatic
            // deserialization.
            let record: csv_models::CSVCard = result?;
            cards.push(record);
        }
        let cards = retrieval
            .bulk_search_cards(
                cards
                    .iter()
                    .map(|c| (c.set_code.clone(), c.collector_number.clone()))
                    .collect(),
            )
            .await?;
        println!("{cards:?}");
        // retrieval.get_cards_by_ids(cards.iter().map(|c| c))
        // retrieve actual cards from retrieval
        // self.add_collection(new_based_on_filename)
        // for card in cards {
        // self.add_card_to_collection(cards)
        // sender.send(progress)
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use retrieval::SQLiteRetrievalSystem;

    use super::*;

    #[tokio::test]
    async fn migrations_csv_import() {
        let mut s = PersistenceSystem::Database(SQLitePersistenceSystem::new(true, None).unwrap());
        let r = RetrievalSystem::Database(SQLiteRetrievalSystem::new(None).unwrap());
        let cards = s.import_csv("".to_string(), r, None).await.unwrap();
    }
}
