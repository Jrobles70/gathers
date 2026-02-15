use axum::{Router, error_handling::HandleErrorLayer};
use clap::{Parser, ValueEnum};
use persistence::PersistenceSystem;
use retrieval::RetrievalSystem;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower::{BoxError, ServiceBuilder};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::debug;

use crate::collections::collection_routes;
use crate::mtg_api::mtg_routes;

mod collections;
mod mtg_api;

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
        })
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

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Systems {
    Scryfall,
    Sql,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[clap(short, long, default_value = "scryfall")]
    system: Systems,

    #[clap(short, long)]
    port: usize,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let port = args.port;

    // Use default paths but make them configurable
    let storage_db_path = std::env::var("STORAGE_DB_PATH")
        .ok()
        .or_else(|| Some("/home/mihail/.local/share/hometg/DB/storage.db".to_string()));

    let retrieval_db_path = std::env::var("RETRIEVAL_DB_PATH")
        .ok()
        .or_else(|| Some("/home/mihail/.local/share/hometg/DB/AllPrintings.db".to_string()));

    let retrieval = Arc::new(Mutex::new(RetrievalState::new(
        args.system,
        retrieval_db_path,
    )?));
    let storage = Arc::new(Mutex::new(StorageState::new(storage_db_path)?));

    let cors = CorsLayer::permissive();
    let app = Router::new()
        .nest("/mtg", mtg_routes())
        .nest("/collection", collection_routes())
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
