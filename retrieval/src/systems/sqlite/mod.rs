mod models;

use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use ::models::{filters::CardSearchFilters, CardID, CollectorNumber, MagicCard, Set, SetCode};
use models::{SqlCard, SqlCardIdentifiers};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use tokio::sync::Mutex;

use crate::{NamedRetrievalSystem, RetrievalSystemTrait};

impl NamedRetrievalSystem for MagicSQLiteRetrievalSystem {}

#[derive(Debug, Clone)]
pub struct MagicSQLiteRetrievalSystem {
    connection: Arc<tokio::sync::Mutex<Connection>>,
    db_path: String,
}

impl MagicSQLiteRetrievalSystem {
    pub fn new(db_path: Option<String>) -> eyre::Result<Self> {
        let path = db_path.unwrap_or_else(|| "../data/testPrintings.db".to_string());
        Ok(Self {
            connection: Arc::new(Mutex::new(Connection::open(path.clone())?)),
            db_path: path,
        })
    }
}

impl RetrievalSystemTrait for MagicSQLiteRetrievalSystem {
    async fn search_cards(
        &self,
        filters: CardSearchFilters,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> eyre::Result<Vec<MagicCard>> {
        let conn = self.connection.lock().await;
        let mut query =
            "SELECT a.uuid, a.name, a.setCode, a.rarity, a.artist, a.colorIdentity, a.text, b.scryfallId, a.number, a.subtypes, a.supertypes, a.types FROM cards as a JOIN cardIdentifiers as b ON a.uuid = b.uuid"
                .to_string();
        let mut conditions = Vec::new();
        let mut params = Vec::new();

        let mut i = 1;
        if let Some(name) = &filters.name {
            if !name.is_empty() {
                conditions.push(format!("a.name LIKE ?{i}"));
                params.push(format!("%{name}%"));
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
                params.push(format!("%{artist}%"));
                i += 1;
            }
        }
        if let Some(text) = &filters.text {
            if !text.is_empty() {
                conditions.push(format!("a.text LIKE ?{i}"));
                params.push(format!("%{text}%"));
                i += 1;
            }
        }
        if let Some(set_code) = &filters.set_code {
            if !set_code.is_empty() {
                conditions.push(format!("a.setCode LIKE ?{i}"));
                params.push(set_code.to_string());
                i += 1;
            }
        }
        if let Some(rarity) = &filters.rarity {
            conditions.push(format!("a.rarity = ?{i}"));
            params.push(format!("{}", rarity.to_single_string()));
            i += 1;
        }
        if let Some(collector_number) = &filters.collector_number {
            conditions.push(format!("a.number = ?{i}"));
            params.push(format!("{collector_number}"));
            i += 1;
        }
        if let Some(subtype) = &filters.subtype {
            if !subtype.is_empty() {
                conditions.push(format!("a.subtypes LIKE ?{i}"));
                params.push(format!("%{subtype}%"));
                i += 1;
            }
        }
        if let Some(supertype) = &filters.supertype {
            if !supertype.is_empty() {
                conditions.push(format!("a.supertypes LIKE ?{i}"));
                params.push(format!("%{supertype}%"));
                i += 1;
            }
        }
        if let Some(types) = &filters.types {
            if !types.is_empty() {
                conditions.push(format!("a.types LIKE ?{i}"));
                params.push(format!("%{types}%"));
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
                subtype: row.get(9)?,
                supertype: row.get(10)?,
                types: row.get(11)?,
            })
        })?;

        Ok(user_iter
            .filter(|c| c.is_ok())
            .map(|c| c.unwrap().into())
            .collect())
    }

    async fn get_cards_by_ids(&self, ids: Vec<String>) -> eyre::Result<HashMap<String, MagicCard>> {
        let conn = self.connection.lock().await;
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT a.uuid, a.name, a.setCode, a.rarity, a.artist, a.colorIdentity, a.text, b.scryfallId, a.number, a.subtypes, a.supertypes, a.types FROM cards as a JOIN cardIdentifiers as b ON a.uuid = b.uuid WHERE a.uuid IN ({})", placeholders
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
                subtype: row.get(9)?,
                supertype: row.get(10)?,
                types: row.get(11)?,
            })
        })?;
        Ok(iter
            .flatten()
            .map(|c| (c.clone().id, c.clone().into()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use ::models::CardColour;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_new_with_none() {
        let system = MagicSQLiteRetrievalSystem::new(None);
        assert!(system.is_ok());
        let system = system.unwrap();
        assert!(!system.db_path.is_empty());
    }

    #[tokio::test]
    async fn test_new_with_custom_path() {
        let temp_dir = TempDir::new().unwrap();
        let custom_path = temp_dir.path().join("test.db");
        let system =
            MagicSQLiteRetrievalSystem::new(Some(custom_path.to_string_lossy().to_string()));
        assert!(system.is_ok());
        let system = system.unwrap();
        assert_eq!(system.db_path, custom_path.to_string_lossy().to_string());
    }

    #[tokio::test]
    async fn test_search_cards_with_name_filter() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let filters = CardSearchFilters {
            name: Some("Goblin King".to_string()),
            ..Default::default()
        };
        let result = system.search_cards(filters, None, None).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(!cards.is_empty());
    }

    #[tokio::test]
    async fn test_search_cards_with_color_identity_filter() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let filters = CardSearchFilters {
            color_identities: Some(vec![CardColour::Black]),
            ..Default::default()
        };
        let result = system.search_cards(filters, None, None).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(!cards.is_empty());
    }

    #[tokio::test]
    async fn test_search_cards_with_artist_filter() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let filters = CardSearchFilters {
            artist: Some("Jason Chan".to_string()),
            ..Default::default()
        };
        let result = system.search_cards(filters, None, None).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(!cards.is_empty());
    }

    #[tokio::test]
    async fn test_search_cards_with_text_filter() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let filters = CardSearchFilters {
            text: Some("destroy target enchantment".to_string()),
            ..Default::default()
        };
        let result = system.search_cards(filters, None, None).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(!cards.is_empty());
    }

    #[tokio::test]
    async fn test_search_cards_with_set_code_filter() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let filters = CardSearchFilters {
            set_code: Some("M20".to_string()),
            ..Default::default()
        };
        let result = system.search_cards(filters, None, None).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(!cards.is_empty());
    }

    #[tokio::test]
    async fn test_search_cards_with_skip_and_limit() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let filters = CardSearchFilters {
            name: Some("Rule of Law".to_string()),
            ..Default::default()
        };
        let result = system.search_cards(filters, Some(6), Some(5)).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(cards.len() <= 5);
    }

    #[tokio::test]
    async fn test_search_cards_empty_result() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let filters = CardSearchFilters {
            name: Some("NonExistentCardXYZ123".to_string()),
            ..Default::default()
        };
        let result = system.search_cards(filters, None, None).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(cards.is_empty());
    }

    #[tokio::test]
    async fn test_get_cards_by_ids() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let ids = vec![
            "0003caab-9ff5-5d1a-bc06-976dd0457f19".to_string(),
            "0005d268-3fd0-5424-bc6b-573ecd713aa1".to_string(),
        ];
        let result = system.get_cards_by_ids(ids).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert_eq!(cards.len(), 2);
    }

    #[tokio::test]
    async fn test_get_cards_by_empty_ids() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let result = system.get_cards_by_ids(vec![]).await;
        assert!(result.is_ok());
        let cards = result.unwrap();
        assert!(cards.is_empty());
    }

    #[tokio::test]
    async fn test_get_sets() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let result = system.get_sets().await;
        assert!(result.is_ok());
        let sets = result.unwrap();
        assert!(!sets.is_empty());
    }

    #[tokio::test]
    async fn test_bulk_search_cards() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let cards = vec![
            (
                SetCode::from_str("TLE").unwrap(),
                CollectorNumber::from_str("12").unwrap(),
            ),
            (
                SetCode::from_str("ARB").unwrap(),
                CollectorNumber::from_str("52").unwrap(),
            ),
        ];
        let result = system.bulk_search_cards(cards).await;
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_bulk_search_cards_empty() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let result = system.bulk_search_cards(vec![]).await;
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_named_retrieval_system_trait() {
        let system = MagicSQLiteRetrievalSystem::new(None).unwrap();
        let name = system.name();
        assert_eq!(
            name,
            "retrieval::systems::sqlite::MagicSQLiteRetrievalSystem"
        );
    }
}
