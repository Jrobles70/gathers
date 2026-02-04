mod sqlite;

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
        provider: String,
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
        provider: String,
    ) -> eyre::Result<CollectionCard> {
        match self {
            PersistenceSystem::Database(d) => {
                d.add_card_to_collection(
                    collection_id,
                    card_uuid,
                    quantity,
                    foil_quantity,
                    time_added,
                    provider,
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
        &mut self,
        filename: String,
        retrieval: RetrievalSystem,
        progress_sender: Option<tokio::sync::watch::Sender<f32>>,
    ) -> eyre::Result<()> {
        let mut rdr = csv::Reader::from_path(filename)?;
        let mut cards = vec![];
        for result in rdr.deserialize() {
            let record: csv_models::CSVCard = result?;
            cards.push(record);
        }
        // TODO: split bulk search by some bucket amount
        let card_ids = retrieval
            .bulk_search_cards(
                cards
                    .iter()
                    .map(|c| (c.set_code.clone(), c.collector_number.clone()))
                    .collect(),
            )
            .await?;

        let cta: Vec<(String, u32, u32)> = card_ids
            .iter()
            .map(|c| {
                let card = cards
                    .iter()
                    .find(|s| s.set_code == c.1 && s.collector_number == c.2);
                match card {
                    Some(s) => (c.0.clone(), s.quantity, s.foil_quantity),
                    None => (c.0.clone(), 0, 0),
                }
            })
            .collect();

        let mut i: f32 = 0.0;
        let total: f32 = cta.len() as f32;
        let now = chrono::Utc::now();
        let time_added = now.to_rfc3339();
        let collection_id = self.add_collection("New Collection".to_string()).await?;
        for c in cta {
            self.add_card_to_collection(
                collection_id.clone(),
                c.0,
                c.1 as i32,
                c.2 as i32,
                time_added.clone(),
                // TODO: retrieval.something
                "".to_string(),
            )
            .await?;

            i += 1.0;
            if let Some(ref sender) = progress_sender {
                sender.send(i / total)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use retrieval::MagicSQLiteRetrievalSystem;

    use super::*;

    #[tokio::test]
    async fn migrations_csv_import() {
        // Test File:
        // Set,CollectorNumber,Quantity,FoilQuantity
        // 10E,16,2,1
        // 10E,17,0,4

        let (sender, receiver) = tokio::sync::watch::channel(0.0);

        let mut s = PersistenceSystem::Database(SQLitePersistenceSystem::new(true, None).unwrap());
        let r = RetrievalSystem::Database(MagicSQLiteRetrievalSystem::new(None).unwrap());
        s.import_csv(
            "/home/mihail/repos/gathers/test.csv".to_string(),
            r,
            Some(sender),
        )
        .await
        .unwrap();

        let collections = s.list_collections().await.unwrap();
        assert_eq!(collections.len(), 2); // Default and the new one
        let new_collection = collections.iter().find(|c| !"Default".eq(*c)).unwrap();

        let card_count = s
            .get_cards_in_collection_count(new_collection.clone())
            .await
            .unwrap();
        assert_eq!(card_count, 2);

        let cards = s
            .get_cards_in_collection_paginated(new_collection.clone(), 0, 10)
            .await
            .unwrap();

        println!("{cards:?}");
        let card = cards
            .iter()
            .find(|c| c.uuid == "d68306e2-9877-5987-84b3-12b8234c8eec")
            .unwrap();
        assert_eq!(card.quantity, 2);
        assert_eq!(card.foil_quantity, 1);

        let card = cards
            .iter()
            .find(|c| c.uuid == "546eac7c-1424-597d-ac13-bf8558e88fe3")
            .unwrap();
        assert_eq!(card.quantity, 0);
        assert_eq!(card.foil_quantity, 4);

        let latest_progress_update = receiver.borrow();
        assert_eq!(*latest_progress_update, 1.0);
    }
}
