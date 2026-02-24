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

use crate::{GathersState, mtg_api::mtg_api_models::APICard};
mod mtg_api_models;

#[derive(Serialize, Debug)]
struct ErrorPayload {
    error: String,
}

fn default_limit() -> usize {
    10
}

pub fn mtg_routes() -> Router<GathersState> {
    #[derive(Deserialize)]
    struct SearchQuery {
        #[serde(default)]
        skip: usize,
        #[serde(default = "default_limit")]
        limit: usize,
    }

    async fn search_mtg_cards(
        State(state): State<GathersState>,
        Query(query): Query<SearchQuery>,
        Json(input): Json<CardSearchFilters>,
    ) -> Result<Json<Vec<APICard>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.0.lock().await.retrieval;

        match ret
            .search_cards(input, query.skip.into(), query.limit.into())
            .await
        {
            Ok(result) => Ok(Json(
                result
                    .iter()
                    .filter_map(|c| match c {
                        Card::Magic(m) => Some(m.clone().into()),
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

    async fn retrieve_cards(
        State(state): State<GathersState>,
        axum_extra::extract::Query(query): axum_extra::extract::Query<RetrieveQuery>,
    ) -> Result<Json<HashMap<String, APICard>>, (StatusCode, Json<ErrorPayload>)> {
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
                        Card::Magic(m) => Some((k, m.into())),
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

    async fn update(
        State(state): State<GathersState>,
    ) -> Result<Json<String>, (StatusCode, Json<ErrorPayload>)> {
        // Get hash of latest version
        // Compare hash with existing saved hash, if any
        // Download new version if hashes don't match
        // Reload state
        let mut ret = state.0.lock().await;
        match ret
            .retrieval
            .update_backend()
            .await
            .and_then(|_| ret.reload_retrieval())
        {
            // match state.0.lock().await.reload_retrieval() {
            Ok(()) => Ok(Json("Update successful".to_string())),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Oof. {e}"),
                }),
            )),
        }
    }

    Router::new()
        .route("/cards/search", post(search_mtg_cards))
        .route("/cards", get(retrieve_cards))
        .route("/sets", get(get_sets))
        .route("/update", get(update))
}
