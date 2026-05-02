use crate::{CollectionCard, CollectionCardsParams, CollectionSortField, PersistenceSystemTrait};
use eyre::eyre;
use include_dir::{Dir, include_dir};
use models::CardID;
use models::CollectionID;
use models::filters::SortOrder;
use rusqlite::{Connection, params};
use rusqlite_migration::Migrations;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;

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
            let path = db_path.unwrap_or_else(|| "storage.db".to_string());
            let path = std::path::Path::new(&path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            Connection::open(path)?
        };
        MIGRATIONS.to_latest(&mut conn)?;
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "foreign_keys", "ON").unwrap();
        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }
}

impl PersistenceSystemTrait for SQLitePersistenceSystem {
    async fn add_collection(&mut self, name: CollectionID) -> eyre::Result<CollectionID> {
        let conn = self.connection.lock().await;
        let query = "INSERT OR IGNORE INTO collection (name, can_remove) VALUES (?1, ?2)";
        conn.execute(query, params![name, true])?;

        Ok(name)
    }

    async fn remove_collection(
        &mut self,
        name: &CollectionID,
        move_to: Option<CollectionID>,
    ) -> eyre::Result<CollectionID> {
        let conn = self.connection.lock().await;

        if let Some(target_collection_id) = move_to {
            let query = "INSERT INTO cards (uuid, collection, quantity, foilquantity, timeadded, timeupdated, provider)
            SELECT uuid, ?1 as collection, quantity, foilquantity, timeadded, strftime('%Y-%m-%dT%H:%M:%SZ', 'now') as timeupdated, provider FROM
	(SELECT uuid, ?2 as collection, quantity, foilquantity, timeadded, provider FROM cards WHERE collection = ?2) WHERE true
            ON CONFLICT (uuid, collection)
            DO UPDATE SET
                quantity = cards.quantity + EXCLUDED.quantity,
                foilquantity = cards.foilquantity + EXCLUDED.foilquantity,
                timeupdated = strftime('%Y-%m-%dT%H:%M:%SZ', 'now');";
            conn.execute(query, params![target_collection_id, name])?;
        }

        let delete_cards_query =
            "DELETE FROM cards WHERE collection IN (SELECT name FROM collection WHERE name = ?1)";
        conn.execute(delete_cards_query, params![name])?;

        let delete_collection_query =
            "DELETE FROM collection WHERE name = ?1 AND can_remove = TRUE";
        conn.execute(delete_collection_query, params![name])?;

        Ok(name.clone())
    }

    async fn move_cards_between_collections(
        &mut self,
        cards: &[CollectionCard],
        to_collection_id: CollectionID,
    ) -> eyre::Result<()> {
        for c in cards {
            if c.quantity == 0 && c.foil_quantity == 0 {
                continue;
            }

            let source_card = self
                .add_card_to_collection(
                    &c.collection,
                    &c.uuid,
                    -c.quantity,
                    -c.foil_quantity,
                    &c.time_added,
                    &c.provider,
                )
                .await?;

            let provider = if source_card.provider.is_empty() {
                c.provider.clone()
            } else {
                source_card.provider.clone()
            };

            self.add_card_to_collection(
                &to_collection_id,
                &c.uuid,
                c.quantity,
                c.foil_quantity,
                &c.time_added,
                &provider,
            )
            .await?;
        }
        Ok(())
    }

    async fn list_collections(&self, filter: Option<String>) -> eyre::Result<Vec<CollectionID>> {
        let conn = self.connection.lock().await;

        let mut collections = Vec::new();
        if let Some(f) = filter {
            let pattern = format!("%{}%", f);
            let mut stmt =
                conn.prepare("SELECT name FROM collection WHERE name LIKE ?1")?;
            let collection_iter = stmt.query_map(params![pattern], |row| {
                let name: String = row.get(0)?;
                Ok(name)
            })?;
            for collection in collection_iter {
                collections.push(collection?);
            }
        } else {
            let mut stmt = conn.prepare("SELECT name FROM collection")?;
            let collection_iter = stmt.query_map(params![], |row| {
                let name: String = row.get(0)?;
                Ok(name)
            })?;
            for collection in collection_iter {
                collections.push(collection?);
            }
        }

        Ok(collections)
    }

    async fn get_cards_in_collection_count(
        &self,
        collection_id: CollectionID,
    ) -> eyre::Result<usize> {
        let conn = self.connection.lock().await;

        let mut stmt = conn.prepare("SELECT COUNT(ALL uuid) FROM cards WHERE collection = ?1")?;
        let mut card_iter = stmt.query_map(params![collection_id], |row| {
            let count: u32 = row.get(0)?;
            Ok(count)
        })?;

        match card_iter.next() {
            Some(count) => match count {
                Ok(c) => Ok(c as usize),
                _ => Err(eyre!("Oh no db")),
            },
            None => Err(eyre!("Oh no")),
        }
    }

    async fn add_card_to_collection(
        &mut self,
        collection_id: &CollectionID,
        card_uuid: &CardID,
        quantity: i32,
        foil_quantity: i32,
        time_added: &str,
        provider: &str,
    ) -> eyre::Result<CollectionCard> {
        let cards = self
            .add_cards_to_collection(
                collection_id,
                &[CollectionCard {
                    uuid: card_uuid.clone(),
                    collection: collection_id.clone(),
                    quantity,
                    foil_quantity,
                    time_added: time_added.to_string(),
                    provider: provider.to_string(),
                }],
            )
            .await?;

        Ok(cards[0].clone())
    }

    async fn add_cards_to_collection(
        &mut self,
        collection_id: &CollectionID,
        cards: &[CollectionCard],
    ) -> eyre::Result<Vec<CollectionCard>> {
        if cards.is_empty() {
            return Ok(vec![]);
        }

        let conn = self.connection.lock().await;

        let placeholders = cards
            .iter()
            .map(|_| "(?, ?, ?, ?, ?, ?, ?)")
            .collect::<Vec<_>>()
            .join(",");
        let mut params = vec![];
        cards.iter().for_each(|c| {
            params.push(c.uuid.clone());
            params.push(collection_id.clone());
            params.push(c.quantity.to_string());
            params.push(c.foil_quantity.to_string());
            params.push(c.time_added.clone());
            params.push(c.time_added.clone()); // timeupdated = timeadded on creation
            params.push(c.provider.clone());
        });
        let query = format!(
            "
INSERT INTO cards (uuid, collection, quantity, foilquantity, timeadded, timeupdated, provider)
VALUES {}
ON CONFLICT (uuid, collection) DO UPDATE SET
 quantity = max(cards.quantity + EXCLUDED.quantity, 0),
 foilquantity = max(cards.foilquantity + EXCLUDED.foilquantity, 0),
 timeupdated = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
RETURNING uuid, collection, quantity, foilquantity, timeadded, provider
",
            placeholders
        );
        let mut stmt = conn.prepare(&query)?;
        let card_iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(CollectionCard {
                uuid: row.get(0)?,
                quantity: row.get(2)?,
                foil_quantity: row.get(3)?,
                time_added: row.get(4)?,
                collection: row.get(1)?,
                provider: row.get(5)?,
            })
        })?;
        let mut cards = Vec::new();
        for card in card_iter.flatten() {
            cards.push(card);
        }
        conn.execute(
            "DELETE FROM cards WHERE quantity = 0 AND foilquantity = 0",
            [],
        )?;

        Ok(cards)
    }

    async fn get_cards_in_collection_paginated(
        &self,
        collection_id: &CollectionID,
        params: CollectionCardsParams,
    ) -> eyre::Result<Vec<CollectionCard>> {
        let conn = self.connection.lock().await;

        let mut conditions = vec!["collection = ?1".to_string()];
        let mut query_params: Vec<String> = vec![collection_id.clone()];
        let mut i = 2;

        if let Some(provider) = &params.provider {
            conditions.push(format!("provider = ?{i}"));
            query_params.push(provider.clone());
            i += 1;
        }

        let sort_col = match &params.sort_by {
            Some(CollectionSortField::Quantity) => "quantity",
            Some(CollectionSortField::FoilQuantity) => "foilquantity",
            Some(CollectionSortField::Provider) => "provider",
            _ => "timeadded",
        };
        let sort_dir = if matches!(&params.sort_order, Some(SortOrder::Desc)) { "DESC" } else { "ASC" };

        let query = format!(
            "SELECT uuid, quantity, foilquantity, timeadded, provider FROM cards WHERE {} ORDER BY {} {} LIMIT ?{} OFFSET ?{}",
            conditions.join(" AND "),
            sort_col,
            sort_dir,
            i,
            i + 1,
        );
        query_params.push(params.limit.to_string());
        query_params.push(params.offset.to_string());

        let mut stmt = conn.prepare(&query)?;
        let collection_id = collection_id.clone();
        let card_iter = stmt.query_map(rusqlite::params_from_iter(query_params.iter()), |row| {
            Ok(CollectionCard {
                uuid: row.get(0)?,
                quantity: row.get(1)?,
                foil_quantity: row.get(2)?,
                time_added: row.get(3)?,
                collection: collection_id.clone(),
                provider: row.get(4)?,
            })
        })?;

        let mut cards = Vec::new();
        for card in card_iter {
            cards.push(card?);
        }

        Ok(cards)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT: &str = "Default";
    const OLD_TIME: &str = "2023-01-01T00:00:00Z";

    #[test]
    fn migrations_test() {
        assert!(MIGRATIONS.validate().is_ok());
    }

    async fn get_time_updated(
        persistence: &SQLitePersistenceSystem,
        collection_id: &str,
        card_uuid: &str,
    ) -> Option<String> {
        let conn = persistence.connection.lock().await;
        conn.query_row(
            "SELECT timeupdated FROM cards WHERE collection = ?1 AND uuid = ?2",
            params![collection_id, card_uuid],
            |row| row.get(0),
        )
        .ok()
    }

    async fn add_card_to_collection(
        persistence: &mut SQLitePersistenceSystem,
        collection_id: &CollectionID,
        card_id: &CardID,
        quantity: i32,
        foil_quantity: i32,
    ) -> CardID {
        persistence
            .add_card_to_collection(
                collection_id,
                card_id,
                quantity,
                foil_quantity,
                OLD_TIME,
                "",
            )
            .await
            .unwrap();
        card_id.clone()
    }

    #[tokio::test]
    async fn test_remove_collection_can_be_removed() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection that can be removed
        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add cards to the collection
        add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), 5, 2).await;

        // Try to remove the collection (should be removed because can_remove is true by default)
        p.remove_collection(&collection_id, None).await.unwrap();

        // Verify collection was removed (because it can be removed)
        let collections = p.list_collections(None).await.unwrap();
        assert!(!collections.contains(&collection_id));
    }

    #[tokio::test]
    async fn test_remove_default_collection_with_move_to() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a regular collection
        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add cards to the regular collection
        add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), 5, 2).await;

        // Add a card to the Default collection
        let cid =
            add_card_to_collection(&mut p, &DEFAULT.into(), &"default_card".to_string(), 3, 1)
                .await;

        // Try to remove the Default collection (should not be removed because can_remove is false)
        // But cards should still be moved to the test collection
        p.remove_collection(&DEFAULT.into(), Some(collection_id.clone()))
            .await
            .unwrap();

        // Verify Default collection still exists
        let collections = p.list_collections(None).await.unwrap();
        assert!(collections.contains(&DEFAULT.into()));

        // Verify cards were moved to collection 1
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 2); // Should have both cards now

        // Verify that card quantities are correct (default_card should have been added to existing card)
        let default_card = cards.iter().find(|c| c.uuid == cid).unwrap();
        assert_eq!(default_card.quantity, 3);
        assert_eq!(default_card.foil_quantity, 1);

        // Verify cards were moved away from Default
        let cards = p
            .get_cards_in_collection_paginated(&DEFAULT.into(), CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_collection_with_none_move_to() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection
        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add cards to the collection
        add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), 5, 2).await;

        // Remove collection with move_to = None (should delete cards and collection)
        p.remove_collection(&collection_id, None).await.unwrap();

        // Verify collection was removed
        let collections = p.list_collections(None).await.unwrap();
        assert!(!collections.contains(&collection_id));

        // Verify no cards remain in the collection
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_collection_management() {
        // Create a new persistence system
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection
        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();
        assert!(!collection_id.is_empty());

        // List collections
        let collections = p.list_collections(None).await.unwrap();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"Test Collection".to_string()));
        assert!(collections.contains(&DEFAULT.into()));

        // Add another collection
        let collection_id2 = p
            .add_collection("Another Collection".to_string())
            .await
            .unwrap();
        assert!(!collection_id2.is_empty());

        // List collections again
        let collections = p.list_collections(None).await.unwrap();
        assert_eq!(collections.len(), 3);
        assert!(collections.contains(&"Test Collection".to_string()));
        assert!(collections.contains(&"Another Collection".to_string()));

        // Remove a collection
        let result = p
            .remove_collection(&"Test Collection".to_string(), None)
            .await
            .unwrap();
        assert!(!result.is_empty());

        // List collections after removal
        let collections = p.list_collections(None).await.unwrap();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&DEFAULT.into()));
        assert!(collections.contains(&"Another Collection".to_string()));
    }

    #[tokio::test]
    async fn test_add_card_to_collection() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection
        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add a card to the collection
        let cid = add_card_to_collection(&mut p, &collection_id, &"12345".to_string(), 2, 1).await;

        // Verify the card was added
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, cid);
        assert_eq!(cards[0].quantity, 2);
        assert_eq!(cards[0].foil_quantity, 1);

        // Add more of the same card
        add_card_to_collection(&mut p, &collection_id, &cid, 3, 2).await;

        // Verify the quantities were updated
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, cid);
        assert_eq!(cards[0].quantity, 5); // 2 + 3
        assert_eq!(cards[0].foil_quantity, 3); // 1 + 2

        // Add negative quantities to reduce card amounts
        add_card_to_collection(&mut p, &collection_id, &cid, -3, -1).await;

        // Verify the quantities were updated
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].uuid, cid);
        assert_eq!(cards[0].quantity, 2); // 5 - 3
        assert_eq!(cards[0].foil_quantity, 2); // 3 - 1

        // Remove all quantities of a card (both regular and foil)
        add_card_to_collection(&mut p, &collection_id, &cid, -2, -2).await;

        // Verify the card was removed from collection (both quantities are 0)
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_get_cards_in_collection_paginated() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add a collection
        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add multiple cards to the collection
        for i in 0..10 {
            add_card_to_collection(&mut p, &collection_id, &(1000 + i).to_string(), 1, 0).await;
        }

        // Test pagination - get first page (limit 5, offset 0)
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 5))
            .await
            .unwrap();
        assert_eq!(cards.len(), 5);
        assert_eq!(cards[0].uuid, "1000".to_string());
        assert_eq!(cards[1].uuid, "1001".to_string());
        assert_eq!(cards[2].uuid, "1002".to_string());
        assert_eq!(cards[3].uuid, "1003".to_string());
        assert_eq!(cards[4].uuid, "1004".to_string());

        // Test pagination - get second page (limit 5, offset 5)
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(5, 5))
            .await
            .unwrap();
        assert_eq!(cards.len(), 5);
        assert_eq!(cards[0].uuid, "1005".to_string());
        assert_eq!(cards[1].uuid, "1006".to_string());
        assert_eq!(cards[2].uuid, "1007".to_string());
        assert_eq!(cards[3].uuid, "1008".to_string());
        assert_eq!(cards[4].uuid, "1009".to_string());

        // Test pagination - get page with less items than limit
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(8, 5))
            .await
            .unwrap();
        assert_eq!(cards.len(), 2); // Only 2 items left
        assert_eq!(cards[0].uuid, "1008".to_string());
        assert_eq!(cards[1].uuid, "1009".to_string());

        // Test pagination - offset beyond available items
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(20, 5))
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_collection_that_cant_be_removed() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();
        add_card_to_collection(&mut p, &collection_id, &"12345".to_string(), 5, 3).await;
        let c = p
            .get_cards_in_collection_count(DEFAULT.into())
            .await
            .unwrap();
        assert_eq!(c, 0);

        add_card_to_collection(&mut p, &DEFAULT.into(), &"12346".to_string(), 2, 8).await;
        let c = p
            .get_cards_in_collection_count(DEFAULT.into())
            .await
            .unwrap();
        assert_eq!(c, 1);

        let cards = p
            .get_cards_in_collection_paginated(&DEFAULT.into(), CollectionCardsParams::new(0, 5))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(p.list_collections(None).await.unwrap().len(), 2);
        p.remove_collection(&DEFAULT.into(), None).await.unwrap();
        assert_eq!(p.list_collections(None).await.unwrap().len(), 2);
        p.remove_collection(&collection_id, None).await.unwrap();
        assert_eq!(p.list_collections(None).await.unwrap().len(), 1);
        let cards = p
            .get_cards_in_collection_paginated(&DEFAULT.into(), CollectionCardsParams::new(0, 5))
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_collection_with_move_to() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        // Add two collections
        let collection1_id = p.add_collection("Collection 1".to_string()).await.unwrap();
        let collection2_id = p.add_collection("Collection 2".to_string()).await.unwrap();

        // Add cards to the first collection
        let cid1 =
            add_card_to_collection(&mut p, &collection1_id, &"card1".to_string(), 5, 2).await;
        let cid2 =
            add_card_to_collection(&mut p, &collection1_id, &"card2".to_string(), 3, 1).await;

        // Verify cards are in collection 1
        let cards1 = p
            .get_cards_in_collection_paginated(&collection1_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards1.len(), 2);

        // Remove collection 1 and move cards to collection 2
        let result = p
            .remove_collection(&collection1_id, Some(collection2_id.clone()))
            .await
            .unwrap();
        assert_eq!(result, collection1_id); // Should return the removed collection ID

        // Verify collection 1 is gone
        let collections = p.list_collections(None).await.unwrap();
        assert!(!collections.contains(&collection1_id));

        // Verify cards are now in collection 2
        let cards2 = p
            .get_cards_in_collection_paginated(&collection2_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards2.len(), 2);

        // Verify the card quantities are correct
        let card1 = cards2.iter().find(|c| c.uuid == cid1).unwrap();
        assert_eq!(card1.quantity, 5);
        assert_eq!(card1.foil_quantity, 2);

        let card2 = cards2.iter().find(|c| c.uuid == cid2).unwrap();
        assert_eq!(card2.quantity, 3);
        assert_eq!(card2.foil_quantity, 1);

        // Verify collection 2 still exists
        assert!(collections.contains(&collection2_id));
    }

    #[tokio::test]
    async fn test_move_cards_between_collections() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        let cid = add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), 5, 2).await;
        add_card_to_collection(&mut p, &DEFAULT.into(), &"default_card".to_string(), 3, 1).await;

        let cards = p
            .get_cards_in_collection_paginated(&DEFAULT.to_string(), CollectionCardsParams::new(0, 100))
            .await
            .unwrap();

        assert_eq!(cards.len(), 1);

        p.move_cards_between_collections(
            &[CollectionCard {
                uuid: cid.clone(),
                quantity: 4,
                foil_quantity: 0,
                time_added: "".to_string(),
                collection: collection_id.clone(),
                provider: "".to_string(),
            }],
            DEFAULT.to_string(),
        )
        .await
        .unwrap();

        let collections = p.list_collections(None).await.unwrap();
        assert!(collections.contains(&DEFAULT.to_string()));

        let cards = p
            .get_cards_in_collection_paginated(&DEFAULT.to_string(), CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 2);

        let card1 = cards.iter().find(|c| c.uuid == cid).unwrap();
        assert_eq!(card1.quantity, 4);
        assert_eq!(card1.foil_quantity, 0);

        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);

        let card1 = cards.iter().find(|c| c.uuid == cid).unwrap();
        assert_eq!(card1.quantity, 1);
        assert_eq!(card1.foil_quantity, 2);
    }

    #[tokio::test]
    async fn test_add_cards_to_collection() {
        let mut persistence = SQLitePersistenceSystem::new(true, None).unwrap();

        let collection_id = persistence
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        let time_added = "2023-01-01T00:00:00Z".to_string();

        persistence
            .add_cards_to_collection(
                &collection_id,
                &[
                    CollectionCard {
                        uuid: "12345".to_string(),
                        quantity: 2,
                        foil_quantity: 1,
                        time_added: time_added.clone(),
                        provider: "".to_string(),
                        collection: collection_id.clone(),
                    },
                    CollectionCard {
                        uuid: "12346".to_string(),
                        quantity: 5,
                        foil_quantity: 0,
                        time_added: time_added.clone(),
                        provider: "".to_string(),
                        collection: collection_id.clone(),
                    },
                ],
            )
            .await
            .unwrap();

        let cards = persistence
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 2);
        let card = cards.iter().find(|c| c.uuid == "12345").unwrap();
        assert_eq!(card.quantity, 2);
        assert_eq!(card.foil_quantity, 1);

        let card = cards.iter().find(|c| c.uuid == "12346").unwrap();
        assert_eq!(card.quantity, 5);
        assert_eq!(card.foil_quantity, 0);

        persistence
            .add_cards_to_collection(
                &collection_id,
                &[CollectionCard {
                    uuid: "12345".to_string(),
                    quantity: 3,
                    foil_quantity: 2,
                    time_added: time_added.clone(),
                    provider: "".to_string(),
                    collection: collection_id.clone(),
                }],
            )
            .await
            .unwrap();

        let cards = persistence
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 2);
        let card = cards.iter().find(|c| c.uuid == "12345").unwrap();
        assert_eq!(card.uuid, "12345".to_string());
        assert_eq!(card.quantity, 5); // 2 + 3
        assert_eq!(card.foil_quantity, 3); // 1 + 2

        persistence
            .add_cards_to_collection(
                &collection_id,
                &[
                    CollectionCard {
                        uuid: "12345".to_string(),
                        quantity: -3,
                        foil_quantity: -1,
                        time_added: time_added.clone(),
                        provider: "".to_string(),
                        collection: collection_id.clone(),
                    },
                    CollectionCard {
                        uuid: "12346".to_string(),
                        quantity: 5,
                        foil_quantity: 0,
                        time_added: time_added.clone(),
                        provider: "".to_string(),
                        collection: collection_id.clone(),
                    },
                ],
            )
            .await
            .unwrap();

        let cards = persistence
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 2);
        let card = cards.iter().find(|c| c.uuid == "12345").unwrap();
        assert_eq!(card.quantity, 2); // 5 - 3
        assert_eq!(card.foil_quantity, 2); // 3 - 1

        let card = cards.iter().find(|c| c.uuid == "12346").unwrap();
        assert_eq!(card.quantity, 10);
        assert_eq!(card.foil_quantity, 0);
    }

    #[tokio::test]
    async fn test_list_collections_with_filter() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        p.add_collection("Test Alpha".to_string()).await.unwrap();
        p.add_collection("Test Beta".to_string()).await.unwrap();
        p.add_collection("Gamma".to_string()).await.unwrap();

        // Filter matching two collections
        let collections = p.list_collections(Some("Test".to_string())).await.unwrap();
        assert_eq!(collections.len(), 2);
        assert!(collections.contains(&"Test Alpha".to_string()));
        assert!(collections.contains(&"Test Beta".to_string()));

        // Filter matching exactly one collection
        let collections = p.list_collections(Some("Alpha".to_string())).await.unwrap();
        assert_eq!(collections.len(), 1);
        assert!(collections.contains(&"Test Alpha".to_string()));

        // Filter matching none
        let collections = p
            .list_collections(Some("XYZ_NOMATCH".to_string()))
            .await
            .unwrap();
        assert!(collections.is_empty());

        // None filter returns all (Default + 3 added)
        let collections = p.list_collections(None).await.unwrap();
        assert_eq!(collections.len(), 4);
    }

    #[tokio::test]
    async fn test_quantity_floor_cannot_go_negative() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Add 3 regular, 2 foil
        add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), 3, 2).await;

        // Over-subtract regular (−100 against 3), subtract 1 foil
        add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), -100, -1).await;

        // Regular floors at 0; foil = 2 − 1 = 1; card still exists
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].quantity, 0);
        assert_eq!(cards[0].foil_quantity, 1);

        // Remove remaining foil; both quantities hit 0 → card deleted
        add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), 0, -1).await;
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_move_cards_between_collections_skips_zero_quantity() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        add_card_to_collection(&mut p, &collection_id, &"card1".to_string(), 5, 2).await;

        // Attempt to move a card with both quantities = 0 — should be a no-op
        p.move_cards_between_collections(
            &[CollectionCard {
                uuid: "card1".to_string(),
                quantity: 0,
                foil_quantity: 0,
                time_added: "2023-01-01T00:00:00Z".to_string(),
                collection: collection_id.clone(),
                provider: "".to_string(),
            }],
            DEFAULT.to_string(),
        )
        .await
        .unwrap();

        // Source collection unchanged
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].quantity, 5);
        assert_eq!(cards[0].foil_quantity, 2);

        // Default collection untouched
        let default_cards = p
            .get_cards_in_collection_paginated(&DEFAULT.to_string(), CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(default_cards.len(), 0);
    }

    #[tokio::test]
    async fn test_remove_collection_move_to_merges_quantities() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let col1 = p.add_collection("Collection 1".to_string()).await.unwrap();
        let col2 = p.add_collection("Collection 2".to_string()).await.unwrap();

        // Same card exists in both collections
        add_card_to_collection(&mut p, &col1, &"shared_card".to_string(), 3, 1).await;
        add_card_to_collection(&mut p, &col2, &"shared_card".to_string(), 2, 4).await;

        // Only-in-col1 card
        add_card_to_collection(&mut p, &col1, &"unique_card".to_string(), 5, 0).await;

        // Remove col1, moving its cards into col2
        p.remove_collection(&col1, Some(col2.clone()))
            .await
            .unwrap();

        let collections = p.list_collections(None).await.unwrap();
        assert!(!collections.contains(&col1));

        let cards = p
            .get_cards_in_collection_paginated(&col2, CollectionCardsParams::new(0, 100))
            .await
            .unwrap();
        assert_eq!(cards.len(), 2);

        // Shared card quantities should be merged
        let shared = cards.iter().find(|c| c.uuid == "shared_card").unwrap();
        assert_eq!(shared.quantity, 5); // 3 + 2
        assert_eq!(shared.foil_quantity, 5); // 1 + 4

        // Unique card moved as-is
        let unique = cards.iter().find(|c| c.uuid == "unique_card").unwrap();
        assert_eq!(unique.quantity, 5);
        assert_eq!(unique.foil_quantity, 0);
    }

    #[tokio::test]
    async fn test_add_collection_duplicate_name_is_idempotent() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        p.add_collection("My Collection".to_string()).await.unwrap();

        // Same name again — should succeed and return the same name
        let result = p.add_collection("My Collection".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "My Collection");

        // "Default" is seeded by migrations; re-adding it must also succeed
        let result = p.add_collection(DEFAULT.to_string()).await;
        assert!(result.is_ok());

        // Total collection count must not have grown
        let collections = p.list_collections(None).await.unwrap();
        assert_eq!(collections.len(), 2); // Default + My Collection
    }

    /// Regression test: moving a card with an explicit provider must preserve
    /// that provider in the destination collection, even when only some copies
    /// are moved (source row survives).
    #[tokio::test]
    async fn test_move_partial_preserves_provider() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let col_a = p.add_collection("Collection A".to_string()).await.unwrap();
        let col_b = p.add_collection("Collection B".to_string()).await.unwrap();

        // Add card with a known provider
        p.add_card_to_collection(
            &col_a,
            &"card1".to_string(),
            5,
            2,
            "2023-01-01T00:00:00Z",
            "mtg",
        )
        .await
        .unwrap();

        // Move 3 regular copies, 0 foil
        p.move_cards_between_collections(
            &[CollectionCard {
                uuid: "card1".to_string(),
                quantity: 3,
                foil_quantity: 0,
                time_added: "2023-01-01T00:00:00Z".to_string(),
                collection: col_a.clone(),
                provider: "".to_string(), // simulates the API sending empty provider
            }],
            col_b.clone(),
        )
        .await
        .unwrap();

        // Source: 2 regular, 2 foil remain
        let src_cards = p
            .get_cards_in_collection_paginated(&col_a, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(src_cards.len(), 1);
        assert_eq!(src_cards[0].quantity, 2);
        assert_eq!(src_cards[0].foil_quantity, 2);

        // Destination: 3 regular, 0 foil — provider must be "mtg", not ""
        let dst_cards = p
            .get_cards_in_collection_paginated(&col_b, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(dst_cards.len(), 1);
        assert_eq!(dst_cards[0].quantity, 3);
        assert_eq!(dst_cards[0].foil_quantity, 0);
        assert_eq!(dst_cards[0].provider, "mtg");
    }

    /// Regression test: moving ALL copies of a card (source row deleted) must
    /// still write the correct provider in the destination.
    #[tokio::test]
    async fn test_move_all_copies_preserves_provider() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let col_a = p.add_collection("Collection A".to_string()).await.unwrap();
        let col_b = p.add_collection("Collection B".to_string()).await.unwrap();

        p.add_card_to_collection(
            &col_a,
            &"card1".to_string(),
            4,
            1,
            "2023-01-01T00:00:00Z",
            "riftbound",
        )
        .await
        .unwrap();

        // Move all copies; source row is deleted after subtract
        p.move_cards_between_collections(
            &[CollectionCard {
                uuid: "card1".to_string(),
                quantity: 4,
                foil_quantity: 1,
                time_added: "2023-01-01T00:00:00Z".to_string(),
                collection: col_a.clone(),
                provider: "".to_string(), // simulates the API sending empty provider
            }],
            col_b.clone(),
        )
        .await
        .unwrap();

        // Source must be empty
        let src_cards = p
            .get_cards_in_collection_paginated(&col_a, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(src_cards.len(), 0);

        // Destination has the card with the correct provider
        let dst_cards = p
            .get_cards_in_collection_paginated(&col_b, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(dst_cards.len(), 1);
        assert_eq!(dst_cards[0].quantity, 4);
        assert_eq!(dst_cards[0].foil_quantity, 1);
        assert_eq!(dst_cards[0].provider, "riftbound");
    }

    /// Regression test: moving a card to the same collection it is already in
    /// must be a no-op (no data loss, no provider corruption).
    #[tokio::test]
    async fn test_move_same_collection_is_noop() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let col = p.add_collection("My Collection".to_string()).await.unwrap();

        p.add_card_to_collection(
            &col,
            &"card1".to_string(),
            5,
            2,
            "2023-01-01T00:00:00Z",
            "mtg",
        )
        .await
        .unwrap();

        // Move card to the same collection (simulates the UI bug where
        // destinationCollection stays as the current collection)
        p.move_cards_between_collections(
            &[CollectionCard {
                uuid: "card1".to_string(),
                quantity: 5,
                foil_quantity: 2,
                time_added: "2023-01-01T00:00:00Z".to_string(),
                collection: col.clone(),
                provider: "".to_string(),
            }],
            col.clone(),
        )
        .await
        .unwrap();

        // Card must still be present with original quantities and correct provider
        let cards = p
            .get_cards_in_collection_paginated(&col, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].quantity, 5);
        assert_eq!(cards[0].foil_quantity, 2);
        assert_eq!(cards[0].provider, "mtg");
    }

    #[tokio::test]
    async fn test_add_cards_to_collection_empty_slice() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();

        let collection_id = p
            .add_collection("Test Collection".to_string())
            .await
            .unwrap();

        // Empty slice should succeed and return an empty vec (not malformed SQL)
        let result = p
            .add_cards_to_collection(&collection_id, &[])
            .await
            .unwrap();
        assert!(result.is_empty());

        // Collection should still be empty
        let cards = p
            .get_cards_in_collection_paginated(&collection_id, CollectionCardsParams::new(0, 10))
            .await
            .unwrap();
        assert_eq!(cards.len(), 0);
    }

    #[tokio::test]
    async fn test_timeupdated_equals_timeadded_on_create() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"card1".to_string(), 2, 1, OLD_TIME, "")
            .await
            .unwrap();

        let time_updated = get_time_updated(&p, &col, "card1").await.unwrap();
        assert_eq!(time_updated, OLD_TIME);
    }

    #[tokio::test]
    async fn test_timeupdated_changes_on_quantity_modification() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"card1".to_string(), 2, 1, OLD_TIME, "")
            .await
            .unwrap();

        // Modify quantity — timeupdated should become current time, not OLD_TIME
        p.add_card_to_collection(&col, &"card1".to_string(), 3, 0, OLD_TIME, "")
            .await
            .unwrap();

        let time_updated = get_time_updated(&p, &col, "card1").await.unwrap();
        assert_ne!(time_updated, OLD_TIME);
    }

    #[tokio::test]
    async fn test_timeupdated_changes_on_foil_quantity_modification() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"card1".to_string(), 2, 1, OLD_TIME, "")
            .await
            .unwrap();

        p.add_card_to_collection(&col, &"card1".to_string(), 0, -1, OLD_TIME, "")
            .await
            .unwrap();

        let time_updated = get_time_updated(&p, &col, "card1").await.unwrap();
        assert_ne!(time_updated, OLD_TIME);
    }

    #[tokio::test]
    async fn test_timeupdated_updated_on_move_source() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col_a = p.add_collection("Collection A".to_string()).await.unwrap();
        let col_b = p.add_collection("Collection B".to_string()).await.unwrap();

        p.add_card_to_collection(&col_a, &"card1".to_string(), 5, 2, OLD_TIME, "")
            .await
            .unwrap();

        p.move_cards_between_collections(
            &[CollectionCard {
                uuid: "card1".to_string(),
                quantity: 3,
                foil_quantity: 1,
                time_added: OLD_TIME.to_string(),
                collection: col_a.clone(),
                provider: "".to_string(),
            }],
            col_b.clone(),
        )
        .await
        .unwrap();

        // Source row quantity was reduced — timeupdated must reflect the move
        let time_updated = get_time_updated(&p, &col_a, "card1").await.unwrap();
        assert_ne!(time_updated, OLD_TIME);
    }

    #[tokio::test]
    async fn test_timeupdated_updated_on_move_destination_existing_card() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col_a = p.add_collection("Collection A".to_string()).await.unwrap();
        let col_b = p.add_collection("Collection B".to_string()).await.unwrap();

        p.add_card_to_collection(&col_a, &"card1".to_string(), 5, 2, OLD_TIME, "")
            .await
            .unwrap();
        // card1 also already exists in col_b — move hits the UPDATE path
        p.add_card_to_collection(&col_b, &"card1".to_string(), 1, 0, OLD_TIME, "")
            .await
            .unwrap();

        p.move_cards_between_collections(
            &[CollectionCard {
                uuid: "card1".to_string(),
                quantity: 2,
                foil_quantity: 1,
                time_added: OLD_TIME.to_string(),
                collection: col_a.clone(),
                provider: "".to_string(),
            }],
            col_b.clone(),
        )
        .await
        .unwrap();

        let time_updated = get_time_updated(&p, &col_b, "card1").await.unwrap();
        assert_ne!(time_updated, OLD_TIME);
    }

    #[tokio::test]
    async fn test_timeupdated_updated_on_remove_collection_merge() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col_a = p.add_collection("Collection A".to_string()).await.unwrap();
        let col_b = p.add_collection("Collection B".to_string()).await.unwrap();

        p.add_card_to_collection(&col_a, &"card1".to_string(), 3, 1, OLD_TIME, "")
            .await
            .unwrap();
        // card1 exists in both collections to exercise the ON CONFLICT UPDATE path
        p.add_card_to_collection(&col_b, &"card1".to_string(), 2, 0, OLD_TIME, "")
            .await
            .unwrap();

        p.remove_collection(&col_a, Some(col_b.clone()))
            .await
            .unwrap();

        let time_updated = get_time_updated(&p, &col_b, "card1").await.unwrap();
        assert_ne!(time_updated, OLD_TIME);
    }

    #[tokio::test]
    async fn test_collection_sort_by_quantity_asc() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"card_a".to_string(), 5, 0, OLD_TIME, "").await.unwrap();
        p.add_card_to_collection(&col, &"card_b".to_string(), 1, 0, OLD_TIME, "").await.unwrap();
        p.add_card_to_collection(&col, &"card_c".to_string(), 3, 0, OLD_TIME, "").await.unwrap();

        let params = CollectionCardsParams {
            offset: 0,
            limit: 10,
            sort_by: Some(CollectionSortField::Quantity),
            sort_order: Some(SortOrder::Asc),
            provider: None,
        };
        let cards = p.get_cards_in_collection_paginated(&col, params).await.unwrap();

        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].quantity, 1);
        assert_eq!(cards[1].quantity, 3);
        assert_eq!(cards[2].quantity, 5);
    }

    #[tokio::test]
    async fn test_collection_sort_by_quantity_desc() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"card_a".to_string(), 5, 0, OLD_TIME, "").await.unwrap();
        p.add_card_to_collection(&col, &"card_b".to_string(), 1, 0, OLD_TIME, "").await.unwrap();
        p.add_card_to_collection(&col, &"card_c".to_string(), 3, 0, OLD_TIME, "").await.unwrap();

        let params = CollectionCardsParams {
            offset: 0,
            limit: 10,
            sort_by: Some(CollectionSortField::Quantity),
            sort_order: Some(SortOrder::Desc),
            provider: None,
        };
        let cards = p.get_cards_in_collection_paginated(&col, params).await.unwrap();

        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].quantity, 5);
        assert_eq!(cards[1].quantity, 3);
        assert_eq!(cards[2].quantity, 1);
    }

    #[tokio::test]
    async fn test_collection_sort_by_foil_quantity_desc() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"card_a".to_string(), 1, 10, OLD_TIME, "").await.unwrap();
        p.add_card_to_collection(&col, &"card_b".to_string(), 1, 2, OLD_TIME, "").await.unwrap();
        p.add_card_to_collection(&col, &"card_c".to_string(), 1, 7, OLD_TIME, "").await.unwrap();

        let params = CollectionCardsParams {
            offset: 0,
            limit: 10,
            sort_by: Some(CollectionSortField::FoilQuantity),
            sort_order: Some(SortOrder::Desc),
            provider: None,
        };
        let cards = p.get_cards_in_collection_paginated(&col, params).await.unwrap();

        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].foil_quantity, 10);
        assert_eq!(cards[1].foil_quantity, 7);
        assert_eq!(cards[2].foil_quantity, 2);
    }

    #[tokio::test]
    async fn test_collection_filter_by_provider() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"mtg1".to_string(), 1, 0, OLD_TIME, "MagicSQLite").await.unwrap();
        p.add_card_to_collection(&col, &"mtg2".to_string(), 2, 0, OLD_TIME, "MagicSQLite").await.unwrap();
        p.add_card_to_collection(&col, &"rb1".to_string(), 1, 0, OLD_TIME, "RiftboundSQLite").await.unwrap();

        let params = CollectionCardsParams {
            offset: 0,
            limit: 10,
            sort_by: None,
            sort_order: None,
            provider: Some("MagicSQLite".to_string()),
        };
        let cards = p.get_cards_in_collection_paginated(&col, params).await.unwrap();

        assert_eq!(cards.len(), 2);
        assert!(cards.iter().all(|c| c.provider == "MagicSQLite"));
    }

    #[tokio::test]
    async fn test_collection_filter_by_provider_no_match() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"card1".to_string(), 1, 0, OLD_TIME, "MagicSQLite").await.unwrap();

        let params = CollectionCardsParams {
            offset: 0,
            limit: 10,
            sort_by: None,
            sort_order: None,
            provider: Some("PokemonSQLite".to_string()),
        };
        let cards = p.get_cards_in_collection_paginated(&col, params).await.unwrap();

        assert!(cards.is_empty());
    }

    #[tokio::test]
    async fn test_collection_filter_and_sort_combined() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"mtg_high".to_string(), 5, 0, OLD_TIME, "MagicSQLite").await.unwrap();
        p.add_card_to_collection(&col, &"mtg_low".to_string(), 1, 0, OLD_TIME, "MagicSQLite").await.unwrap();
        p.add_card_to_collection(&col, &"rb1".to_string(), 99, 0, OLD_TIME, "RiftboundSQLite").await.unwrap();

        let params = CollectionCardsParams {
            offset: 0,
            limit: 10,
            sort_by: Some(CollectionSortField::Quantity),
            sort_order: Some(SortOrder::Asc),
            provider: Some("MagicSQLite".to_string()),
        };
        let cards = p.get_cards_in_collection_paginated(&col, params).await.unwrap();

        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0].uuid, "mtg_low");
        assert_eq!(cards[1].uuid, "mtg_high");
    }

    #[tokio::test]
    async fn test_collection_sort_by_provider() {
        let mut p = SQLitePersistenceSystem::new(true, None).unwrap();
        let col = p.add_collection("Col".to_string()).await.unwrap();

        p.add_card_to_collection(&col, &"z_card".to_string(), 1, 0, OLD_TIME, "ZProvider").await.unwrap();
        p.add_card_to_collection(&col, &"a_card".to_string(), 1, 0, OLD_TIME, "AProvider").await.unwrap();
        p.add_card_to_collection(&col, &"m_card".to_string(), 1, 0, OLD_TIME, "MProvider").await.unwrap();

        let params = CollectionCardsParams {
            offset: 0,
            limit: 10,
            sort_by: Some(CollectionSortField::Provider),
            sort_order: Some(SortOrder::Asc),
            provider: None,
        };
        let cards = p.get_cards_in_collection_paginated(&col, params).await.unwrap();

        assert_eq!(cards.len(), 3);
        assert_eq!(cards[0].provider, "AProvider");
        assert_eq!(cards[1].provider, "MProvider");
        assert_eq!(cards[2].provider, "ZProvider");
    }
}
