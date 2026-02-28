use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use axum_extra::extract::Query;
use models::{Card, filters::CardSearchFilters};
use reqwest::StatusCode;
use retrieval::RetrievalSystemTrait;
use serde::{Deserialize, Serialize};

use crate::{GathersState, pokemon_api::pokemon_api_models::APIPokemonCard};
mod pokemon_api_models;

#[derive(Serialize, Debug)]
struct ErrorPayload {
    error: String,
}

fn default_limit() -> usize {
    24
}

pub fn pokemon_routes() -> Router<GathersState> {
    #[derive(Deserialize)]
    struct SearchQuery {
        #[serde(default)]
        skip: usize,
        #[serde(default = "default_limit")]
        limit: usize,
    }

    async fn search_pokemon_cards(
        State(state): State<GathersState>,
        Query(query): Query<SearchQuery>,
        Json(input): Json<CardSearchFilters>,
    ) -> Result<Json<Vec<APIPokemonCard>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.0.lock().await.retrieval;

        match ret
            .search_cards(input, query.skip.into(), query.limit.into())
            .await
        {
            Ok(result) => Ok(Json(
                result
                    .iter()
                    .filter_map(|c| match c {
                        Card::Pokemon(p) => Some(p.clone().into()),
                        _ => None,
                    })
                    .collect(),
            )),
            Err(_) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: "Failed to search cards".into(),
                }),
            )),
        }
    }

    #[derive(Deserialize)]
    struct RetrieveQuery {
        #[serde(default)]
        ids: Vec<String>,
    }

    async fn retrieve_pokemon_cards(
        State(state): State<GathersState>,
        axum_extra::extract::Query(query): axum_extra::extract::Query<RetrieveQuery>,
    ) -> Result<Json<HashMap<String, APIPokemonCard>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.0.lock().await.retrieval;

        ret.get_cards_by_ids(query.ids)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: "Oof".into(),
                    }),
                )
            })
            .map(|d| {
                d.into_iter()
                    .filter_map(|(k, v)| match v {
                        Card::Pokemon(p) => Some((k, p.into())),
                        _ => None,
                    })
                    .collect()
            })
            .map(Json)
    }

    async fn get_sets(
        State(state): State<GathersState>,
    ) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.0.lock().await.retrieval;

        ret.get_sets()
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: "Oof".into(),
                    }),
                )
            })
            .map(|s| Json(s.iter().map(|s| s.code.clone()).collect()))
    }

    Router::new()
        .route("/cards/search", post(search_pokemon_cards))
        .route("/cards", get(retrieve_pokemon_cards))
        .route("/sets", get(get_sets))
}
