use aide::axum::{ApiRouter, routing::get};
use aide::openapi::{Info, OpenApi};
use aide::swagger::Swagger;
use axum::http::StatusCode;
use axum::{Extension, Json, error_handling::HandleErrorLayer, extract::State};
use clap::{Parser, ValueEnum};
use persistence::PersistenceSystem;
use retrieval::{RetrievalSystem, RetrievalSystemTrait};
use schemars::JsonSchema;
use serde::Serialize;
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

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ErrorPayload {
    pub error: String,
}

/// Convenience alias for the standard API error response.
pub type ApiError = (StatusCode, Json<ErrorPayload>);

#[derive(Debug, Clone, serde::Serialize, JsonSchema)]
pub struct SystemInfo {
    /// Primary system (for backward compatibility).
    pub system: Systems,
    /// All configured systems.
    pub systems: Vec<Systems>,
}

type GathersState = (Arc<Mutex<RetrievalState>>, Arc<Mutex<StorageState>>);

#[derive(Debug, Clone)]
pub struct RetrievalState {
    pub mtg: Option<RetrievalSystem>,
    pub riftbound: Option<RetrievalSystem>,
    pub pokemon: Option<RetrievalSystem>,
    /// Which MTG system variant is active (Scryfall or Sql), for reload support.
    mtg_system_type: Option<Systems>,
    mtg_db_path: Option<String>,
    riftbound_db_path: Option<String>,
    pokemon_db_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct StorageState {
    storage: PersistenceSystem,
    _storage_db_path: Option<String>,
}

impl RetrievalState {
    pub fn new(
        systems: Vec<Systems>,
        mtg_db_path: Option<String>,
        riftbound_db_path: Option<String>,
        pokemon_db_path: Option<String>,
    ) -> eyre::Result<RetrievalState> {
        let mut state = RetrievalState {
            mtg: None,
            riftbound: None,
            pokemon: None,
            mtg_system_type: None,
            mtg_db_path: mtg_db_path.clone(),
            riftbound_db_path: riftbound_db_path.clone(),
            pokemon_db_path: pokemon_db_path.clone(),
        };

        for system in systems {
            let db_path = match system {
                Systems::Scryfall | Systems::Sql => mtg_db_path.clone(),
                Systems::RiftboundSql => riftbound_db_path.clone(),
                Systems::PokemonSql => pokemon_db_path.clone(),
            };
            let retrieval = Self::new_retrieval(system, db_path)?;
            match system {
                Systems::Scryfall | Systems::Sql => {
                    state.mtg = Some(retrieval);
                    state.mtg_system_type = Some(system);
                }
                Systems::RiftboundSql => state.riftbound = Some(retrieval),
                Systems::PokemonSql => state.pokemon = Some(retrieval),
            }
        }

        Ok(state)
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

    pub fn active_systems(&self) -> Vec<Systems> {
        let mut systems = Vec::new();
        if let Some(s) = self.mtg_system_type {
            systems.push(s);
        }
        if self.riftbound.is_some() {
            systems.push(Systems::RiftboundSql);
        }
        if self.pokemon.is_some() {
            systems.push(Systems::PokemonSql);
        }
        systems
    }

    /// Returns the primary system for webui compatibility.
    /// Prefers MTG, then Riftbound, then Pokemon.
    pub fn primary_system(&self) -> Systems {
        if let Some(s) = self.mtg_system_type {
            s
        } else if self.riftbound.is_some() {
            Systems::RiftboundSql
        } else {
            Systems::PokemonSql
        }
    }

    pub async fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            system: self.primary_system(),
            systems: self.active_systems(),
        }
    }

    pub fn require_mtg(&self) -> Result<&RetrievalSystem, ApiError> {
        self.mtg.as_ref().ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorPayload {
                    error: "MTG system not configured".into(),
                }),
            )
        })
    }

    pub fn require_riftbound(&self) -> Result<&RetrievalSystem, ApiError> {
        self.riftbound.as_ref().ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorPayload {
                    error: "Riftbound system not configured".into(),
                }),
            )
        })
    }

    pub fn require_pokemon(&self) -> Result<&RetrievalSystem, ApiError> {
        self.pokemon.as_ref().ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorPayload {
                    error: "Pokemon system not configured".into(),
                }),
            )
        })
    }

    pub fn reload_mtg(&mut self) -> eyre::Result<()> {
        if let Some(system) = self.mtg_system_type {
            self.mtg = Some(Self::new_retrieval(system, self.mtg_db_path.clone())?);
        }
        Ok(())
    }

    pub fn reload_riftbound(&mut self) -> eyre::Result<()> {
        if self.riftbound.is_some() {
            self.riftbound = Some(Self::new_retrieval(
                Systems::RiftboundSql,
                self.riftbound_db_path.clone(),
            )?);
        }
        Ok(())
    }

    pub fn reload_pokemon(&mut self) -> eyre::Result<()> {
        if self.pokemon.is_some() {
            self.pokemon = Some(Self::new_retrieval(
                Systems::PokemonSql,
                self.pokemon_db_path.clone(),
            )?);
        }
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

#[derive(
    Copy, Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord, Debug, serde::Serialize, JsonSchema,
)]
pub enum Systems {
    Scryfall,
    Sql,
    RiftboundSql,
    PokemonSql,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Retrieval systems to enable. May be specified multiple times.
    /// At least one is required. Supported values: scryfall, sql, riftbound-sql, pokemon-sql.
    #[clap(short, long, required = true, num_args = 1..)]
    system: Vec<Systems>,

    #[clap(short, long)]
    port: usize,
}

async fn get_system_info(
    State(state): State<GathersState>,
) -> Result<Json<SystemInfo>, (axum::http::StatusCode, Json<String>)> {
    let ret = state.0.lock().await;
    Ok(Json(ret.get_system_info().await))
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl axum::response::IntoResponse {
    Json(api)
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let port = args.port;

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let gathers_dir = std::path::Path::new(&home).join(".local/share/gathers");

    let storage_db_path = std::env::var("STORAGE_DB_PATH").ok().or_else(|| {
        Some(
            gathers_dir
                .join("DB/storage.db")
                .to_string_lossy()
                .into_owned(),
        )
    });

    let mtg_db_path = std::env::var("MTG_DB_PATH").ok().or_else(|| {
        Some(
            gathers_dir
                .join("DB/AllPrintings.db")
                .to_string_lossy()
                .into_owned(),
        )
    });

    let riftbound_db_path = std::env::var("RIFTBOUND_DB_PATH").ok().or_else(|| {
        Some(
            gathers_dir
                .join("DB/riftbound.db")
                .to_string_lossy()
                .into_owned(),
        )
    });

    let pokemon_db_path = std::env::var("POKEMON_DB_PATH").ok().or_else(|| {
        Some(
            gathers_dir
                .join("DB/pokemon.db")
                .to_string_lossy()
                .into_owned(),
        )
    });

    if std::env::var("GATHERS_NO_AUTO_UPDATE").is_err() {
        for system in &args.system {
            match system {
                Systems::Sql => {
                    if let Some(ref path) = mtg_db_path
                        && !std::path::Path::new(path).exists()
                    {
                        retrieval::download_mtg_db(path).await?;
                    }
                }
                Systems::RiftboundSql => {
                    if let Some(ref path) = riftbound_db_path
                        && !std::path::Path::new(path).exists()
                    {
                        let temp =
                            RetrievalState::new_retrieval(*system, riftbound_db_path.clone())?;
                        temp.update_backend().await?;
                    }
                }
                _ => unimplemented!(),
            }
        }
    }

    let retrieval = Arc::new(Mutex::new(RetrievalState::new(
        args.system,
        mtg_db_path,
        riftbound_db_path,
        pokemon_db_path,
    )?));
    let storage = Arc::new(Mutex::new(StorageState::new(storage_db_path)?));

    let mut api = OpenApi {
        info: Info {
            title: "GatheRs API".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let cors = CorsLayer::permissive();
    let app = ApiRouter::new()
        .nest("/mtg", mtg_routes())
        .nest("/riftbound", riftbound_routes())
        .nest("/pokemon", pokemon_routes())
        .nest("/collection", collection_routes())
        .api_route("/system", get(get_system_info))
        .route("/api.json", axum::routing::get(serve_api))
        .route("/swagger", Swagger::new("/api.json").axum_route())
        .finish_api(&mut api)
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
        .layer(Extension(api))
        .with_state((retrieval, storage));

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    debug!(port = ?port, "Started server" );

    axum::serve(listener, app).await?;

    Ok(())
}
