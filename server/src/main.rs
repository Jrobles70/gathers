use axum::{Json, Router, error_handling::HandleErrorLayer, extract::State};
use clap::{Parser, ValueEnum};
use persistence::PersistenceSystem;
use retrieval::{RetrievalSystem, RetrievalSystemTrait};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower::{BoxError, ServiceBuilder};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::debug;

use crate::collections::collection_routes;
use crate::mtg_api::mtg_routes;
use crate::pokemon_api::pokemon_routes;
use crate::riftbound_api::riftbound_routes;

mod collections;
mod mtg_api;
mod pokemon_api;
mod riftbound_api;

#[derive(Debug, Clone, serde::Serialize)]
pub struct SystemInfo {
    pub system: Systems,
}

type GathersState = (Arc<Mutex<RetrievalState>>, Arc<Mutex<StorageState>>);

#[derive(Debug, Clone)]
pub struct RetrievalState {
    retrieval: RetrievalSystem,
    system: Systems,
    retrieval_db_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StorageState {
    storage: PersistenceSystem,
    _storage_db_path: Option<String>,
}

impl RetrievalState {
    pub fn new(system: Systems, retrieval_db_path: Option<String>) -> eyre::Result<RetrievalState> {
        Ok(RetrievalState {
            retrieval: RetrievalState::new_retrieval(system, retrieval_db_path.clone())?,
            system,
            retrieval_db_path,
        })
    }

    pub fn new_retrieval(
        system: Systems,
        retrieval_db_path: Option<String>,
    ) -> eyre::Result<RetrievalSystem> {
        Ok(match system {
            Systems::Scryfall => {
                RetrievalSystem::ScryfallRetrievalSystem(retrieval::ScryfallRetrievalSystem::new()?)
            }
            Systems::Sql => RetrievalSystem::MagicSQLiteRetrievalSystem(
                retrieval::MagicSQLiteRetrievalSystem::new(retrieval_db_path.clone())?,
            ),
            Systems::RiftboundSql => RetrievalSystem::RiftboundSQLiteRetrievalSystem(
                retrieval::RiftboundSQLiteRetrievalSystem::new(retrieval_db_path.clone())?,
            ),
            Systems::PokemonSql => RetrievalSystem::PokemonSQLiteRetrievalSystem(
                retrieval::PokemonSQLiteRetrievalSystem::new(retrieval_db_path.clone())?,
            ),
        })
    }

    pub async fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            system: self.system,
        }
    }

    pub fn reload_retrieval(&mut self) -> eyre::Result<()> {
        self.retrieval =
            RetrievalState::new_retrieval(self.system, self.retrieval_db_path.clone())?;
        Ok(())
    }
}

impl StorageState {
    pub fn new(storage_db_path: Option<String>) -> eyre::Result<StorageState> {
        Ok(StorageState {
            storage: PersistenceSystem::SQLitePersistenceSystem(
                persistence::SQLitePersistenceSystem::new(false, storage_db_path.clone())?,
            ),
            _storage_db_path: storage_db_path,
        })
    }
}

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Debug, serde::Serialize)]
pub enum Systems {
    Scryfall,
    Sql,
    RiftboundSql,
    PokemonSql,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[clap(short, long, default_value = "scryfall")]
    system: Systems,

    #[clap(short, long)]
    port: usize,
}

async fn get_system_info(
    State(state): State<GathersState>,
) -> Result<Json<SystemInfo>, (axum::http::StatusCode, Json<String>)> {
    let ret = state.0.lock().await;
    Ok(Json(ret.get_system_info().await))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let port = args.port;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let gathers_dir = std::path::Path::new(&home).join(".local/share/gathers");

    let storage_db_path = std::env::var("STORAGE_DB_PATH")
        .ok()
        .or_else(|| Some(gathers_dir.join("DB/storage.db").to_string_lossy().into_owned()));

    let retrieval_db_path = std::env::var("RETRIEVAL_DB_PATH")
        .ok()
        .or_else(|| Some(gathers_dir.join("DB/AllPrintings.db").to_string_lossy().into_owned()));

    if std::env::var("GATHERS_NO_AUTO_UPDATE").is_err() {
        match args.system {
            Systems::Sql => {
                if let Some(ref path) = retrieval_db_path
                    && !std::path::Path::new(path).exists()
                {
                    retrieval::download_mtg_db(path).await?;
                }
            }
            Systems::RiftboundSql => {
                if let Some(ref path) = retrieval_db_path
                    && !std::path::Path::new(path).exists()
                {
                    let temp =
                        RetrievalState::new_retrieval(args.system, retrieval_db_path.clone())?;
                    temp.update_backend().await?;
                }
            }
            _ => {}
        }
    }

    let retrieval = Arc::new(Mutex::new(RetrievalState::new(
        args.system,
        retrieval_db_path,
    )?));
    let storage = Arc::new(Mutex::new(StorageState::new(storage_db_path)?));

    let cors = CorsLayer::permissive();
    let app = Router::new()
        .nest("/mtg", mtg_routes())
        .nest("/riftbound", riftbound_routes())
        .nest("/pokemon", pokemon_routes())
        .nest("/collection", collection_routes())
        .route("/system", axum::routing::get(get_system_info))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(axum::http::StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {error}"),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .layer(cors)
        .with_state((retrieval, storage));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    debug!(port = ?port, "Started server" );

    axum::serve(listener, app).await?;

    Ok(())
}
