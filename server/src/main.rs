use axum::{error_handling::HandleErrorLayer, Router};
use clap::{Parser, ValueEnum};
use persistence::PersistenceSystem;
use retrieval::RetrievalSystem;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing::debug;

use crate::collections::collection_routes;
use crate::mtg_api::mtg_routes;

mod collections;
mod mtg_api;

type GathersState = Arc<Mutex<AppState>>;

#[derive(Debug, Clone)]
struct AppState {
    retrieval: RetrievalSystem,
    storage: PersistenceSystem,
    system: Systems,
    retrieval_db_path: Option<String>,
    storage_db_path: Option<String>,
}

impl AppState {
    pub fn new(
        system: Systems,
        retrieval_db_path: Option<String>,
        storage_db_path: Option<String>,
    ) -> eyre::Result<Self> {
        Ok(AppState {
            retrieval: AppState::new_retrieval(system, retrieval_db_path.clone())?,
            storage: PersistenceSystem::Database(persistence::SQLitePersistenceSystem::new(
                false,
                storage_db_path.clone(),
            )?),
            system,
            retrieval_db_path,
            storage_db_path,
        })
    }

    pub fn new_retrieval(
        system: Systems,
        retrieval_db_path: Option<String>,
    ) -> eyre::Result<RetrievalSystem> {
        Ok(match system {
            Systems::Scryfall => {
                RetrievalSystem::Scryfall(retrieval::ScryfallRetrievalSystem::new()?)
            }
            Systems::Sql => RetrievalSystem::Database(retrieval::MagicSQLiteRetrievalSystem::new(
                retrieval_db_path.clone(),
            )?),
        })
    }

    pub fn reload_retrieval(&mut self) -> eyre::Result<()> {
        self.retrieval = AppState::new_retrieval(self.system, self.retrieval_db_path.clone())?;
        Ok(())
    }
}

#[derive(Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Systems {
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

    let state = Arc::new(Mutex::new(AppState::new(
        args.system,
        retrieval_db_path,
        storage_db_path,
    )?));

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
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    debug!(port = ?port, "Started server" );

    axum::serve(listener, app).await?;

    Ok(())
}
