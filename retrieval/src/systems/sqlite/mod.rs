mod models;

use std::{collections::HashMap, sync::Arc};

use ::models::{filters::CardSearchFilters, Card, CardID, CollectorNumber, Set, SetCode};
use models::{SqlCard, SqlCardIdentifiers};
use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::RetrievalSystemTrait;

#[derive(Debug, Clone)]
pub struct SQLiteRetrievalSystem {
    connection: Arc<tokio::sync::Mutex<Connection>>,
}

impl SQLiteRetrievalSystem {
    pub fn new(db_path: Option<String>) -> eyre::Result<Self> {
        let path = db_path
            .unwrap_or_else(|| "/home/mihail/.local/share/hometg/DB/AllPrintings.db".to_string());
        Ok(Self {
            connection: Arc::new(Mutex::new(Connection::open(path)?)),
        })
    }
}

#[async_trait::async_trait]
impl RetrievalSystemTrait for SQLiteRetrievalSystem {
    async fn search_cards(
        &self,
        filters: CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<Card>> {
        let conn = self.connection.lock().await;
        let mut query =
            "SELECT a.uuid, a.name, a.setCode, a.rarity, a.artist, a.colorIdentity, a.text, b.scryfallId, a.number FROM cards as a JOIN cardIdentifiers as b ON a.uuid = b.uuid"
                .to_string();
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        let mut i = 1;
        if let Some(name) = &filters.name {
            if !name.is_empty() {
                conditions.push(format!("a.name LIKE ?{i}"));
                params.push(format!("%{}%", name.as_str()));
                i += 1;
            }
        }
        if let Some(colours) = &filters.color_identities {
            for colour in colours {
                conditions.push(format!("a.colorIdentity LIKE ?{i}"));
                params.push(format!("%{colour}%"));
                i += 1;
            }
        }
        if let Some(artist) = &filters.artist {
            if !artist.is_empty() {
                conditions.push(format!("a.artist LIKE ?{i}"));
                params.push(format!("%{}%", artist.as_str()));
                i += 1;
            }
        }
        if let Some(text) = &filters.text {
            if !text.is_empty() {
                conditions.push(format!("a.text LIKE ?{i}"));
                params.push(text.to_string());
                i += 1;
            }
        }
        if let Some(set_code) = &filters.set_code {
            if !set_code.is_empty() {
                conditions.push(format!("a.setCode LIKE ?{i}"));
                params.push(set_code.to_string());
                // i += 1;
            }
        }
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        if let Some(limit) = limit {
            query.push_str(format!(" LIMIT {limit} COLLATE NOCASE").as_str());
        } else {
            query.push_str(" LIMIT 1 COLLATE NOCASE");
        }
        if let Some(skip) = skip {
            query.push_str(format!(" OFFSET {skip}").as_str())
        }
        println!("{}", query);
        let mut stmt = conn.prepare(&query)?;
        let user_iter = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
            Ok(SqlCard {
                id: row.get(0)?,
                name: row.get(1)?,
                set_code: row.get(2)?,
                color_identity: row.get(5)?,
                text: row.get(6)?,
                rarity: row.get(3)?,
                artist: row.get(4)?,
                card_identifiers: SqlCardIdentifiers {
                    scryfall_id: row.get(7)?,
                    id: row.get(0)?,
                },
                collector_number: row.get(8)?,
            })
        })?;

        Ok(user_iter
            .filter(|c| c.is_ok())
            .map(|c| c.unwrap().into())
            .collect())
    }

    async fn get_cards_by_ids(&self, ids: Vec<String>) -> eyre::Result<HashMap<String, Card>> {
        let conn = self.connection.lock().await;
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT a.uuid, a.name, a.setCode, a.rarity, a.artist, a.colorIdentity, a.text, b.scryfallId, a.number FROM cards as a JOIN cardIdentifiers as b ON a.uuid = b.uuid WHERE a.uuid IN ({})", placeholders
            );
        let mut stmt = conn.prepare(&query)?;
        let iter = stmt.query_map(rusqlite::params_from_iter(ids), |row| {
            Ok(SqlCard {
                id: row.get(0)?,
                name: row.get(1)?,
                set_code: row.get(2)?,
                color_identity: row.get(5)?,
                text: row.get(6)?,
                rarity: row.get(3)?,
                artist: row.get(4)?,
                card_identifiers: SqlCardIdentifiers {
                    scryfall_id: row.get(7)?,
                    id: row.get(0)?,
                },
                collector_number: row.get(8)?,
            })
        })?;
        Ok(iter
            .flatten()
            .map(|c| (c.clone().id, c.clone().into()))
            .collect())
    }

    async fn get_sets(&self) -> eyre::Result<Vec<Set>> {
        let conn = self.connection.lock().await;
        let query = "SELECT DISTINCT setCode FROM cards LIMIT 20".to_string();
        let mut stmt = conn.prepare(&query)?;
        let iter = stmt.query_map([], |row| {
            Ok(Set {
                code: row.get(0)?,
                name: "".to_string(),
            })
        })?;
        Ok(iter.flatten().collect())
    }

    async fn bulk_search_cards(
        &self,
        cards: Vec<(SetCode, CollectorNumber)>,
    ) -> eyre::Result<Vec<CardID>> {
        let conn = self.connection.lock().await;
        // TODO: sanitise inputs
        let placeholders = cards
            .iter()
            .map(|c| format!("('{}', '{}')", c.0, c.1))
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "SELECT uuid FROM cards WHERE (setCode, number) IN (VALUES {});",
            placeholders
        );
        println!("{query}");
        let mut stmt = conn.prepare(&query)?;
        let iter = stmt.query_map([], |row| row.get(0))?;
        Ok(iter.flatten().map(|c: String| c).collect())
    }
}
