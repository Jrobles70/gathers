use crate::PersistenceSystemTrait;
use include_dir::{include_dir, Dir};
use rusqlite::{params, Connection};
use rusqlite_migration::Migrations;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use uuid::Uuid;

static MIGRATIONS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/migrations");
static MIGRATIONS: LazyLock<Migrations<'static>> =
    LazyLock::new(|| Migrations::from_directory(&MIGRATIONS_DIR).expect("AAAAH!"));

#[derive(Debug, Clone)]
pub struct SQLitePersistenceSystem {
    connection: Arc<tokio::sync::Mutex<Connection>>,
}

impl SQLitePersistenceSystem {
    pub fn new(in_memory: bool) -> eyre::Result<Self> {
        let mut conn = if in_memory {
            Connection::open(":memory:")?
        } else {
            Connection::open("/home/mihail/.local/share/hometg/DB/storage.db")?
        };
        MIGRATIONS.to_latest(&mut conn)?;
        println!("Ran migrations!");
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait::async_trait]
impl PersistenceSystemTrait for SQLitePersistenceSystem {
    async fn add_collection(&mut self, name: String) -> eyre::Result<String> {
        let collection_id = Uuid::new_v4().to_string();

        let conn = self.connection.lock().await;
        let query = "INSERT INTO collection (id, name) VALUES (?1, ?2)";
        conn.execute(query, params![collection_id, name])?;

        Ok(collection_id)
    }

    async fn remove_collection(&mut self, name: String) -> eyre::Result<String> {
        let conn = self.connection.lock().await;

        let delete_cards_query =
            "DELETE FROM cards WHERE collection IN (SELECT id FROM collection WHERE name = ?1)";
        conn.execute(delete_cards_query, params![name])?;

        let delete_collection_query = "DELETE FROM collection WHERE name = ?1";
        conn.execute(delete_collection_query, params![name])?;

        Ok("Collection removed successfully".to_string())
    }

    async fn list_collections(&self) -> eyre::Result<Vec<String>> {
        let conn = self.connection.lock().await;

        let mut stmt = conn.prepare("SELECT name FROM collection")?;
        let collection_iter = stmt.query_map(params![], |row| {
            let name: String = row.get(0)?;
            Ok(name)
        })?;

        let mut collections = Vec::new();
        for collection in collection_iter {
            collections.push(collection?);
        }

        Ok(collections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        assert!(MIGRATIONS.validate().is_ok());
    }

    #[tokio::test]
    async fn test_collection_management() {
        // Create a new persistence system
        let mut persistence = SQLitePersistenceSystem::new(true).unwrap();

        // Add a collection
        let collection_id = persistence
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();
        assert!(!collection_id.is_empty());

        // List collections
        let collections = persistence.list_collections().await.unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0], "Test Collection");

        // Add another collection
        let collection_id2 = persistence
            .add_collection("Another Collection".to_string())
            .await
            .unwrap();
        assert!(!collection_id2.is_empty());

        // List collections again
        let collections = persistence.list_collections().await.unwrap();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"Test Collection".to_string()));
        assert!(collections.contains(&"Another Collection".to_string()));

        // Remove a collection
        let result = persistence
            .remove_collection("Test Collection".to_string())
            .await
            .unwrap();
        assert!(!result.is_empty());

        // List collections after removal
        let collections = persistence.list_collections().await.unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0], "Another Collection");
    }
}
