use crate::PersistenceSystemTrait;
use eyre::eyre;
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
    pub fn new(in_memory: bool, db_path: Option<String>) -> eyre::Result<Self> {
        let mut conn = if in_memory {
            Connection::open(":memory:")?
        } else {
            // TODO: replace hard-coded path
            let path = db_path
                .unwrap_or_else(|| "/home/mihail/.local/share/hometg/DB/storage.db".to_string());
            Connection::open(path)?
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

        // TODO: handle pagination in case of collection count > 1000
        let mut stmt = conn.prepare("SELECT name FROM collection LIMIT 1000")?;
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

    async fn get_cards_in_collection_count(&self, collection_id: String) -> eyre::Result<usize> {
        let conn = self.connection.lock().await;

        let mut stmt = conn.prepare("SELECT COUNT(ALL uuid) FROM cards WHERE collection = ?1")?;
        let mut card_iter = stmt.query_map(params![collection_id], |row| {
            let count: usize = row.get(0)?;
            Ok(count)
        })?;

        match card_iter.next() {
            Some(count) => match count {
                Ok(c) => Ok(c),
                _ => Err(eyre!("Oh no db")),
            },
            None => Err(eyre!("Oh no")),
        }
    }

    async fn add_card_to_collection(
        &mut self,
        collection_id: String,
        card_uuid: String,
        quantity: i32,
        foil_quantity: i32,
        time_added: String,
    ) -> eyre::Result<CollectionCard> {
        let conn = self.connection.lock().await;

        // First check if card already exists in collection
        let mut stmt = conn.prepare(
            "SELECT quantity, foilquantity, timeadded FROM cards WHERE uuid = ?1 AND collection = ?2",
        )?;
        let existing_card = stmt.query_row(params![card_uuid, collection_id], |row| {
            let quantity: u32 = row.get(0)?;
            let foil_quantity: u32 = row.get(1)?;
            let time: String = row.get(2)?;
            Ok((quantity, foil_quantity, time))
        });

        match existing_card {
            Ok((existing_quantity, existing_foil_quantity, time_added)) => {
                // Card exists, update quantities
                let new_quantity = (existing_quantity as i32 + quantity).max(0) as u32;
                let new_foil_quantity =
                    (existing_foil_quantity as i32 + foil_quantity).max(0) as u32;

                // If both quantities are 0, remove the card from collection
                if new_quantity == 0 && new_foil_quantity == 0 {
                    conn.execute(
                        "DELETE FROM cards WHERE uuid = ?1 AND collection = ?2",
                        params![card_uuid, collection_id],
                    )?;
                    return Ok(CollectionCard {
                        uuid: card_uuid,
                        quantity: 0,
                        foil_quantity: 0,
                        time_added: "".to_string(),
                    });
                } else {
                    // Update the existing card
                    conn.execute(
                        "UPDATE cards SET quantity = ?1, foilquantity = ?2 WHERE uuid = ?3 AND collection = ?4",
                        params![new_quantity, new_foil_quantity, card_uuid, collection_id],
                    )?;

                    return Ok(CollectionCard {
                        uuid: card_uuid,
                        quantity: new_quantity,
                        foil_quantity: new_foil_quantity,
                        time_added,
                    });
                }
            }
            Err(_) => {
                // Card doesn't exist, insert new one
                if quantity > 0 || foil_quantity > 0 {
                    conn.execute(
                        "INSERT INTO cards (uuid, collection, quantity, foilquantity, timeadded) VALUES (?1, ?2, ?3, ?4, ?5)",
                        params![card_uuid, collection_id, quantity.max(0) as u32, foil_quantity.max(0) as u32, time_added],
                    )?;
                }
                return Ok(CollectionCard {
                    uuid: card_uuid,
                    quantity: quantity.max(0) as u32,
                    foil_quantity: foil_quantity.max(0) as u32,
                    time_added,
                });
            }
        }
    }

    async fn get_cards_in_collection_paginated(
        &self,
        collection_id: String,
        offset: usize,
        limit: usize,
    ) -> eyre::Result<Vec<CollectionCard>> {
        let conn = self.connection.lock().await;

        let mut stmt = conn.prepare(
            "SELECT uuid, quantity, foilquantity, timeadded FROM cards WHERE collection = ?1 LIMIT ?2 OFFSET ?3",
        )?;
        let card_iter = stmt.query_map(params![collection_id, limit, offset], |row| {
            let uuid: String = row.get(0)?;
            let quantity: u32 = row.get(1)?;
            let foil_quantity: u32 = row.get(2)?;
            let time_added: String = row.get(3)?;
            Ok(CollectionCard {
                uuid,
                quantity,
                foil_quantity,
                time_added,
            })
        })?;

        let mut cards = Vec::new();
        for card in card_iter {
            cards.push(card?);
        }

        Ok(cards)
    }
}

#[derive(Debug, Clone)]
pub struct CollectionCard {
    pub uuid: String,
    pub quantity: u32,
    pub foil_quantity: u32,
    pub time_added: String,
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
        let mut persistence = SQLitePersistenceSystem::new(true, None).unwrap();

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

    #[tokio::test]
    async fn test_add_card_to_collection() {
        let mut persistence = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection
        let collection_id = persistence
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add a card to the collection
        persistence
            .add_card_to_collection(
                collection_id.clone(),
                12345,
                2,
                1,
                "2023-01-01T00:00:00Z".to_string(),
            )
            .await
            .unwrap();

        // Verify the card was added
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone())
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, 12345);
        assert_eq!(cards[0].quantity, 2);
        assert_eq!(cards[0].foil_quantity, 1);

        // Add more of the same card
        persistence
            .add_card_to_collection(
                collection_id.clone(),
                12345,
                3,
                2,
                "2023-01-01T00:00:00Z".to_string(),
            )
            .await
            .unwrap();

        // Verify the quantities were updated
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone())
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, 12345);
        assert_eq!(cards[0].quantity, 5); // 2 + 3
        assert_eq!(cards[0].foil_quantity, 3); // 1 + 2

        // Add negative quantities to reduce card amounts
        persistence
            .add_card_to_collection(
                collection_id.clone(),
                12345,
                -3,
                -1,
                "2023-01-01T00:00:00Z".to_string(),
            )
            .await
            .unwrap();

        // Verify the quantities were updated
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone())
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, 12345);
        assert_eq!(cards[0].quantity, 2); // 5 - 3
        assert_eq!(cards[0].foil_quantity, 2); // 3 - 1

        // Remove all quantities of a card (both regular and foil)
        persistence
            .add_card_to_collection(
                collection_id.clone(),
                12345,
                -2,
                -2,
                "2023-01-01T00:00:00Z".to_string(),
            )
            .await
            .unwrap();

        // Verify the card was removed from collection (both quantities are 0)
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone())
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_get_cards_in_collection_paginated() {
        let mut persistence = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection
        let collection_id = persistence
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add multiple cards to the collection
        for i in 0..10 {
            persistence
                .add_card_to_collection(
                    collection_id.clone(),
                    1000 + i,
                    1,
                    0,
                    "2023-01-01T00:00:00Z".to_string(),
                )
                .await
                .unwrap();
        }

        // Test pagination - get first page (limit 5, offset 0)
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone(), 0, 5)
            .await
            .unwrap();
        assert_eq!(cards.len(), 5);
        assert_eq!(cards[0].uuid, 1000);
        assert_eq!(cards[1].uuid, 1001);
        assert_eq!(cards[2].uuid, 1002);
        assert_eq!(cards[3].uuid, 1003);
        assert_eq!(cards[4].uuid, 1004);

        // Test pagination - get second page (limit 5, offset 5)
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone(), 5, 5)
            .await
            .unwrap();
        assert_eq!(cards.len(), 5);
        assert_eq!(cards[0].uuid, 1005);
        assert_eq!(cards[1].uuid, 1006);
        assert_eq!(cards[2].uuid, 1007);
        assert_eq!(cards[3].uuid, 1008);
        assert_eq!(cards[4].uuid, 1009);

        // Test pagination - get page with less items than limit
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone(), 8, 5)
            .await
            .unwrap();
        assert_eq!(cards.len(), 2); // Only 2 items left
        assert_eq!(cards[0].uuid, 1008);
        assert_eq!(cards[1].uuid, 1009);

        // Test pagination - offset beyond available items
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone(), 20, 5)
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_add_card_to_collection_with_negative_quantities() {
        let mut persistence = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection
        let collection_id = persistence
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add a card to the collection
        persistence
            .add_card_to_collection(
                collection_id.clone(),
                12345,
                5,
                3,
                "2023-01-01T00:00:00Z".to_string(),
            )
            .await
            .unwrap();

        // Verify the card was added
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone())
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, 12345);
        assert_eq!(cards[0].quantity, 5);
        assert_eq!(cards[0].foil_quantity, 3);

        // Try to reduce quantity with negative amounts
        persistence
            .add_card_to_collection(
                collection_id.clone(),
                12345,
                -2,
                -8,
                "2023-01-01T00:00:00Z".to_string(),
            )
            .await
            .unwrap();

        // Verify quantities were updated
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone())
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, 12345);
        assert_eq!(cards[0].quantity, 3); // 5 - 2
        assert_eq!(cards[0].foil_quantity, 0); // 3 - 1

        // Try to reduce quantities below zero - should clamp at zero
        persistence
            .add_card_to_collection(
                collection_id.clone(),
                12345,
                -10,
                -10,
                "2023-01-01T00:00:00Z".to_string(),
            )
            .await
            .unwrap();

        // Verify quantities were clamped at zero
        let cards = persistence
            .get_cards_in_collection_paginated(collection_id.clone())
            .await
            .unwrap();
        assert_eq!(cards.len(), 0); // Card should be removed when both quantities are 0
    }
}
