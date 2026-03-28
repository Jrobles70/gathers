use std::collections::HashMap;

use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum_extra::extract::Query;
use models::Card;
use retrieval::RetrievalSystemTrait;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    ApiError, ErrorPayload, GathersState, collections::collections_models::APICardSearchFilters,
    mtg_api::mtg_api_models::APICard,
};
pub mod mtg_api_models;

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
    ) -> Result<Json<Vec<APICard>>, ApiError> {
        let guard = state.0.lock().await;
        let ret = guard.require_mtg()?;

        ret.search_cards(input.into(), query.skip.into(), query.limit.into())
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to search cards. {e}"),
                    }),
                )
            })
            .map(|result| {
                Json(
                    result
                        .iter()
                        .filter_map(|c| match c {
                            Card::Magic(p) => Some(p.clone().into()),
                            _ => None,
                        })
                        .collect(),
                )
            })
    }

    #[derive(Deserialize, JsonSchema)]
    struct MagicRetrieveQuery {
        #[serde(default)]
        ids: Vec<String>,
    }

    async fn retrieve_cards(
        State(state): State<GathersState>,
        Query(query): Query<MagicRetrieveQuery>,
    ) -> Result<Json<HashMap<String, APICard>>, ApiError> {
        let guard = state.0.lock().await;
        let ret = guard.require_mtg()?;

        ret.get_cards_by_ids(query.ids)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to retrieve cards. {e}"),
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

    async fn get_sets(State(state): State<GathersState>) -> Result<Json<Vec<String>>, ApiError> {
        let guard = state.0.lock().await;
        let ret = guard.require_mtg()?;

        ret.get_sets()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to get sets. {e}"),
                    }),
                )
            })
            .map(|s| Json(s.iter().map(|s| s.code.clone()).collect()))
    }

    async fn update(State(state): State<GathersState>) -> Result<Json<String>, ApiError> {
        let mut ret = state.0.lock().await;
        let result = {
            let mtg = ret.require_mtg()?;
            mtg.update_backend().await
        };
        match result.and_then(|_| ret.reload_mtg()) {
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
