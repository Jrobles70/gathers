mod models;

use std::{collections::HashMap, sync::Arc};

use ::models::{Card, CardID, CollectorNumber, Set, SetCode, filters::CardSearchFilters};
use models::SqlCard;
use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::{NamedRetrievalSystem, RetrievalSystemTrait};

impl NamedRetrievalSystem for RiftboundSQLiteRetrievalSystem {}

#[derive(Debug, Clone)]
pub struct RiftboundSQLiteRetrievalSystem {
    connection: Arc<tokio::sync::Mutex<Connection>>,
    db_path: String,
}

impl RiftboundSQLiteRetrievalSystem {
    pub fn new(db_path: Option<String>) -> eyre::Result<Self> {
        let path = db_path.unwrap_or_else(|| "../data/riftbound.db".to_string());
        Ok(Self {
            connection: Arc::new(Mutex::new(Connection::open(path.clone())?)),
            db_path: path,
        })
    }
}

impl RetrievalSystemTrait for RiftboundSQLiteRetrievalSystem {
    async fn search_cards(
        &self,
        filters: CardSearchFilters, // TODO
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<Card>> {
        let conn = self.connection.lock().await;
        let mut query =
            "SELECT id, name, set_id, rarity, artists, domains, text, image_url, code FROM cards"
                .to_string();
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        let mut i = 1;
        if let Some(name) = &filters.name
            && !name.is_empty()
        {
            conditions.push(format!("name LIKE ?{i}"));
            params.push(format!("%{name}%"));
            i += 1;
        }
        if let Some(colours) = &filters.color_identities {
            for colour in colours {
                conditions.push(format!("domains LIKE ?{i}"));
                params.push(format!("%{colour}%"));
                i += 1;
            }
        }
        if let Some(artist) = &filters.artist
            && !artist.is_empty()
        {
            conditions.push(format!("artists LIKE ?{i}"));
            params.push(format!("%{artist}%"));
            i += 1;
        }
        if let Some(text) = &filters.text
            && !text.is_empty()
        {
            conditions.push(format!("text LIKE ?{i}"));
            params.push(format!("%{text}%"));
            i += 1;
        }
        if let Some(set_code) = &filters.set_code
            && !set_code.is_empty()
        {
            conditions.push(format!("set_id LIKE ?{i}"));
            params.push(set_code.to_string());
            i += 1;
        }
        if let Some(rarity) = &filters.rarity {
            conditions.push(format!("rarity = ?{i}"));
            params.push(rarity.to_single_string().to_string());
            i += 1;
        }
        if let Some(collector_number) = &filters.collector_number
            && !collector_number.is_empty()
        {
            conditions.push(format!("code = ?{i}"));
            params.push(collector_number.to_string());
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

        let mut stmt = conn.prepare(&query)?;
        let user_iter =
            stmt.query_map(rusqlite::params_from_iter(params.iter()), SqlCard::from_row)?;

        Ok(user_iter
            .filter(|c| c.is_ok())
            .map(|c| Card::Riftbound(c.unwrap().into()))
            .collect())
    }

    async fn get_cards_by_ids(&self, ids: Vec<String>) -> eyre::Result<HashMap<String, Card>> {
        let conn = self.connection.lock().await;
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT id, name, set_id, rarity, artists, domains, text, image, code FROM cards WHERE id IN ({})",
            placeholders
        );
        let mut stmt = conn.prepare(&query)?;
        let iter = stmt.query_map(rusqlite::params_from_iter(ids), SqlCard::from_row)?;
        Ok(iter
            .flatten()
            .map(|c| (c.clone().id, Card::Riftbound(c.clone().into())))
            .collect())
    }

    async fn get_sets(&self) -> eyre::Result<Vec<Set>> {
        let conn = self.connection.lock().await;
        let query = "SELECT DISTINCT set_id FROM cards".to_string();
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
    ) -> eyre::Result<Vec<(SetCode, CollectorNumber, CardID)>> {
        if cards.is_empty() {
            return Ok(vec![]);
        }
        let conn = self.connection.lock().await;
        let placeholders = cards.iter().map(|_| "(?,?)").collect::<Vec<_>>().join(",");
        let mut params = vec![];
        cards.iter().for_each(|c| {
            params.push(c.0.clone());
            params.push(c.1.clone());
        });
        let query = format!(
            "SELECT id, set_id, code FROM cards WHERE (set_id, code) IN (VALUES {});",
            placeholders
        );
        let mut stmt = conn.prepare(&query)?;
        let iter = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        Ok(iter
            .flatten()
            .map(|c: (String, String, String)| c)
            .collect())
    }

    async fn update_backend(&self) -> eyre::Result<bool> {
        todo!("Oh no")
    }
}
