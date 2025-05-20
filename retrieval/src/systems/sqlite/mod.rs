mod models;

use std::sync::Arc;

use ::models::{filters::CardSearchFilters, Card};
use models::{SqlCard, SqlCardIdentifiers};
use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::RetrievalSystemTrait;

#[derive(Debug, Clone)]
pub struct SQLiteRetrievalSystem {
    connection: Arc<tokio::sync::Mutex<Connection>>,
}

impl SQLiteRetrievalSystem {
    pub fn new() -> eyre::Result<Self> {
        Ok(Self {
            connection: Arc::new(Mutex::new(Connection::open(
                "/home/mihail/.local/share/hometg/DB/AllPrintings.db",
            )?)),
        })
    }
}

#[async_trait::async_trait]
impl RetrievalSystemTrait for SQLiteRetrievalSystem {
    async fn get_card(&self, filters: CardSearchFilters) -> eyre::Result<Option<Card>> {
        let conn = self.connection.lock().await;
        let mut query =
            "SELECT uuid, name, setCode, rarity, artist, colorIdentity, text FROM cards"
                .to_string();
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        let mut i = 1;
        if let Some(name) = &filters.name {
            conditions.push(format!("name LIKE ?{i}"));
            params.push(format!("%{}%", name.as_str()));
            i = i + 1;
        }
        if let Some(colours) = &filters.color_identities {
            for colour in colours {
                conditions.push(format!("colorIdentity LIKE ?{i}"));
                params.push(format!("%{colour}%"));
                i = i + 1;
            }
        }
        if let Some(artist) = &filters.artist {
            conditions.push(format!("artist LIKE ?{i}"));
            params.push(format!("%{}%", artist.as_str()));
            i = i + 1;
        }
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        query.push_str(" LIMIT 1 COLLATE NOCASE");
        println!("{query}");
        let mut stmt = conn.prepare(&query)?;
        let mut user_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            println!("Color: {:?}", row);
            Ok(SqlCard {
                id: row.get(0)?,
                name: row.get(1)?,
                set_code: row.get(2)?,
                color_identity: row.get(5)?,
                text: row.get(6)?,
                rarity: row.get(3)?,
                artist: row.get(4)?,
                card_identifiers: SqlCardIdentifiers {
                    scryfall_id: "".to_string(),
                    id: "".to_string(),
                },
            })
        })?;

        match user_iter.next() {
            Some(u) => Ok(Some(u?.into())),
            None => Ok(None),
        }
    }
}
