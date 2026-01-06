use std::collections::HashMap;

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Query;
use models::{filters::CardSearchFilters, Card, Set};
use reqwest::StatusCode;
use retrieval::RetrievalSystemTrait;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::GathersState;

#[derive(Serialize)]
struct ErrorPayload {
    error: String,
}

pub fn mtg_routes() -> Router<GathersState> {
    #[derive(Deserialize)]
    struct SearchQuery {
        #[serde(default)]
        skip: usize,
        #[serde(default)]
        limit: usize,
    }

    async fn search_mtg_cards(
        State(state): State<GathersState>,
        Query(query): Query<SearchQuery>,
        Json(input): Json<CardSearchFilters>,
    ) -> Result<Json<Vec<Card>>, (StatusCode, Json<ErrorPayload>)> {
        debug!("{:?}", input);
        let ret = &state.lock().await.retrieval;

        match ret
            .search_cards(input, query.skip.into(), query.limit.into())
            .await
        {
            Ok(result) => Ok(Json(result)),
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
    ) -> Result<Json<HashMap<String, Card>>, (StatusCode, Json<ErrorPayload>)> {
        println!("{:?}", query.ids);
        let ret = &state.lock().await.retrieval;

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
            .map(Json)
    }

    async fn get_sets(
        State(state): State<GathersState>,
    ) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.lock().await.retrieval;

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
        .route("/cards/search", post(search_mtg_cards))
        .route("/cards", get(retrieve_cards))
        .route("/sets", get(get_sets))
}
