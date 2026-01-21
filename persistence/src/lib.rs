mod sqlite;
pub use crate::sqlite::CollectionCard;
pub use crate::sqlite::SQLitePersistenceSystem;

#[derive(Debug, Clone)]
pub enum PersistenceSystem {
    Database(SQLitePersistenceSystem),
}

#[async_trait::async_trait]
pub trait PersistenceSystemTrait {
    async fn add_collection(&mut self, name: String) -> eyre::Result<String>;

    async fn remove_collection(&mut self, name: String) -> eyre::Result<String>;

    async fn list_collections(&self) -> eyre::Result<Vec<String>>;

    async fn get_cards_in_collection(
        &self,
        collection_id: String,
    ) -> eyre::Result<Vec<CollectionCard>>;

    async fn add_card_to_collection(
        &mut self,
        collection_id: String,
        card_uuid: i64,
        quantity: i32,
        foil_quantity: i32,
        time_added: String,
    ) -> eyre::Result<()>;

    async fn get_cards_in_collection_paginated(
        &self,
        collection_id: String,
        offset: usize,
        limit: usize,
    ) -> eyre::Result<Vec<CollectionCard>>;
}

#[async_trait::async_trait]
impl PersistenceSystemTrait for PersistenceSystem {
    async fn add_collection(&mut self, name: String) -> eyre::Result<String> {
        match self {
            PersistenceSystem::Database(d) => d.add_collection(name).await,
        }
    }

    async fn remove_collection(&mut self, name: String) -> eyre::Result<String> {
        match self {
            PersistenceSystem::Database(d) => d.remove_collection(name).await,
        }
    }

    async fn list_collections(&self) -> eyre::Result<Vec<String>> {
        match self {
            PersistenceSystem::Database(d) => d.list_collections().await,
        }
    }

    async fn get_cards_in_collection(
        &self,
        collection_id: String,
    ) -> eyre::Result<Vec<CollectionCard>> {
        match self {
            PersistenceSystem::Database(d) => d.get_cards_in_collection(collection_id).await,
        }
    }

    async fn get_cards_in_collection_paginated(
        &self,
        collection_id: String,
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
        collection_id: String,
        card_uuid: i64,
        quantity: i32,
        foil_quantity: i32,
        time_added: String,
    ) -> eyre::Result<()> {
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
}
