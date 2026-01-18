use axum::{error_handling::HandleErrorLayer, Router};
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
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let port = 5234;

    // Use default paths but make them configurable
    let storage_db_path = std::env::var("STORAGE_DB_PATH")
        .ok()
        .or_else(|| Some("/home/mihail/.local/share/hometg/DB/storage.db".to_string()));

    let retrieval_db_path = std::env::var("RETRIEVAL_DB_PATH")
        .ok()
        .or_else(|| Some("/home/mihail/.local/share/hometg/DB/AllPrintings.db".to_string()));

    let state = Arc::new(Mutex::new(AppState {
        retrieval: RetrievalSystem::Database(retrieval::SQLiteRetrievalSystem::new(
            retrieval_db_path,
        )?),
        storage: PersistenceSystem::Database(persistence::SQLitePersistenceSystem::new(
            false,
            storage_db_path,
        )?),
    }));

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
