use std::collections::HashMap;

use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Query, State},
};
use models::Card;
use reqwest::StatusCode;
use retrieval::RetrievalSystemTrait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    GathersState, collections::collections_models::APICardSearchFilters,
    mtg_api::mtg_api_models::APICard,
};
pub mod mtg_api_models;

#[derive(Serialize, Debug, JsonSchema)]
struct ErrorPayload {
    error: String,
}

fn default_limit() -> usize {
    10
}

pub fn mtg_routes() -> ApiRouter<GathersState> {
    #[derive(Deserialize, JsonSchema)]
    struct MagicSearchQuery {
        #[serde(default)]
        skip: usize,
        #[serde(default = "default_limit")]
        limit: usize,
    }

    async fn search_mtg_cards(
        State(state): State<GathersState>,
        Query(query): Query<MagicSearchQuery>,
        Json(input): Json<APICardSearchFilters>,
    ) -> Result<Json<Vec<APICard>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.0.lock().await.retrieval;

        match ret
            .search_cards(input.into(), query.skip.into(), query.limit.into())
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

    #[derive(Deserialize, JsonSchema)]
    struct MagicRetrieveQuery {
        #[serde(default)]
        ids: Vec<String>,
    }

    async fn retrieve_cards(
        State(state): State<GathersState>,
        Query(query): Query<MagicRetrieveQuery>,
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
        let mut ret = state.0.lock().await;
        match ret
            .retrieval
            .update_backend()
            .await
            .and_then(|_| ret.reload_retrieval())
        {
            Ok(()) => Ok(Json("Update successful".to_string())),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Oof. {e}"),
                }),
            )),
        }
    }

    ApiRouter::new()
        .api_route("/cards/search", post(search_mtg_cards))
        .api_route("/cards", get(retrieve_cards))
        .api_route("/sets", get(get_sets))
        .api_route("/update", get(update))
}
