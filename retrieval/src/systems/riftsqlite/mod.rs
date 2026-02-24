mod models;

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use ::models::{Card, CardID, CollectorNumber, Set, SetCode, filters::CardSearchFilters};
use models::SqlCard;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
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
            conditions.push(format!("a.name LIKE ?{i}"));
            params.push(format!("%{name}%"));
            i += 1;
        }
        if let Some(colours) = &filters.color_identities {
            for colour in colours {
                conditions.push(format!("a.domains LIKE ?{i}"));
                params.push(format!("%{colour}%"));
                i += 1;
            }
        }
        if let Some(artist) = &filters.artist
            && !artist.is_empty()
        {
            conditions.push(format!("a.artists LIKE ?{i}"));
            params.push(format!("%{artist}%"));
            i += 1;
        }
        if let Some(text) = &filters.text
            && !text.is_empty()
        {
            conditions.push(format!("a.text LIKE ?{i}"));
            params.push(format!("%{text}%"));
            i += 1;
        }
        if let Some(set_code) = &filters.set_code
            && !set_code.is_empty()
        {
            conditions.push(format!("a.set_id LIKE ?{i}"));
            params.push(set_code.to_string());
            i += 1;
        }
        if let Some(rarity) = &filters.rarity {
            conditions.push(format!("a.rarity = ?{i}"));
            params.push(rarity.to_single_string().to_string());
            i += 1;
        }
        if let Some(collector_number) = &filters.collector_number
            && !collector_number.is_empty()
        {
            conditions.push(format!("a.code = ?{i}"));
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
            "SELECT a.uuid, a.name, a.setCode, a.rarity, a.artist, a.colorIdentity, a.text, b.scryfallId, a.number, a.subtypes, a.supertypes, a.types FROM cards as a JOIN cardIdentifiers as b ON a.uuid = b.uuid WHERE a.uuid IN ({})",
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
        let query = "SELECT DISTINCT setCode FROM cards".to_string();
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
            "SELECT uuid, setCode, number FROM cards WHERE (setCode, number) IN (VALUES {});",
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
        async fn download_file(url: &str, path: &Path) -> eyre::Result<()> {
            let response = reqwest::get(url).await?;
            let mut file = fs::File::create(path)?;
            let mut content = io::Cursor::new(response.bytes().await?);
            io::copy(&mut content, &mut file)?;
            println!("Finished download to {file:?}");
            Ok(())
        }

        fn calculate_sha256(path: &Path) -> eyre::Result<String> {
            let data = fs::read(path)?;
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let result = hasher.finalize();
            Ok(hex::encode(result))
        }

        fn read_sha256_from_file(path: &Path) -> eyre::Result<String> {
            let content = fs::read_to_string(path)?;
            Ok(content.trim().to_lowercase())
        }

        let local_file_path = PathBuf::from_str(&self.db_path)?;
        let download_url = "https://mtgjson.com/api/v5/AllPrintings.sqlite";
        let crc_url = "https://mtgjson.com/api/v5/AllPrintings.sqlite.sha256";

        let temp_dir = tempfile::tempdir()?;
        let crc_file_path = temp_dir.path().join("remote_crc.sha");

        download_file(crc_url, &crc_file_path).await?;
        let remote_crc = read_sha256_from_file(&crc_file_path)?;

        let local_crc = if local_file_path.exists() {
            calculate_sha256(&local_file_path)?
        } else {
            String::from("invalid") // Dummy value that won't match
        };

        if remote_crc != local_crc {
            println!(
                "CRC mismatch! Local: {}, Remote: {}. Downloading replacement...",
                local_crc, remote_crc
            );

            tokio::spawn(async move {
                let temp_dir = tempfile::tempdir().expect("Gotta be able to create a temp dir");
                let download_file_path = temp_dir.path().join("downloaded_file");

                println!("Download from {download_url:?} to {download_file_path:?}...");
                download_file(download_url, &download_file_path)
                    .await
                    .and_then(|_| calculate_sha256(&download_file_path))
                    .map(|downloaded_crc| {
                        if downloaded_crc == remote_crc {
                            fs::copy(&download_file_path, local_file_path)
                                .expect("File failed to copy");
                            println!("File replaced successfully.");
                        }
                    })
                    .map_err(|e| println!("Failed to download due to {e}"))
            });

            Ok(true)
        } else {
            println!("CRCs match ({}). No replacement needed.", local_crc);
            Ok(false)
        }
    }
}
