mod sqlite;
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
}
