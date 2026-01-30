mod sqlite;
use models::CardID;
use models::CollectionID;

pub use crate::sqlite::SQLitePersistenceSystem;

#[derive(Debug, Clone)]
pub enum PersistenceSystem {
    Database(SQLitePersistenceSystem),
}

#[derive(Debug, Clone)]
pub struct CollectionCard {
    pub uuid: CardID,
    pub quantity: u32,
    pub foil_quantity: u32,
    pub time_added: String,
}

#[derive(Debug, Clone)]
pub struct Collection {
    pub id: CollectionID,
    pub can_remove: bool,
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
}
