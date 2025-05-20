use axum::{error_handling::HandleErrorLayer, Router};
use mtg_api::mtg_routes;
use retrieval::RetrievalSystem;
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;

mod mtg_api;

type GathersState = Arc<Mutex<AppState>>;

#[derive(Debug, Clone)]
struct AppState {
    retrieval: RetrievalSystem,
}

#[tokio::main]
async fn main() {
    let state = Arc::new(Mutex::new(AppState {
        retrieval: RetrievalSystem::Database(retrieval::SQLiteRetrievalSystem::new().unwrap()),
    }));

    let app = Router::new()
        .nest("/mtg", mtg_routes())
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
