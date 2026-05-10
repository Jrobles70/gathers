use std::collections::HashMap;

use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::http::StatusCode;
use axum::{Json, extract::State};
use axum_extra::extract::Query;
use models::{Card, Set};
use retrieval::RetrievalSystemTrait;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    ApiError, ErrorPayload, GathersState, collections::collections_models::APICardSearchFilters,
    mtg_api::mtg_api_models::APICard,
    prices::{api_price_from_cache, cached_prices_for_scryfall_ids},
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

        let result = ret.search_cards(input.into(), query.skip.into(), query.limit.into())
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to search cards. {e}"),
                    }),
                )
            })?;
        drop(guard);

        let price_cache = cached_prices_for_scryfall_ids(
            &state,
            result.iter().filter_map(|c| match c {
                Card::Magic(m) => Some(m.card_identifiers.scryfall_id.clone()),
                _ => None,
            }),
            0,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to read card prices. {e}"),
                }),
            )
        })?;

        Ok(Json(
            result
                .iter()
                .filter_map(|c| match c {
                    Card::Magic(p) => Some(APICard::from_magic_with_price(
                        p.clone(),
                        price_cache
                            .get(&p.card_identifiers.scryfall_id)
                            .map(api_price_from_cache),
                    )),
                    _ => None,
                })
                .collect(),
        ))
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

        let result = ret.get_cards_by_ids(query.ids)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to retrieve cards. {e}"),
                    }),
                )
            })?;
        drop(guard);

        let price_cache = cached_prices_for_scryfall_ids(
            &state,
            result.values().filter_map(|c| match c {
                Card::Magic(m) => Some(m.card_identifiers.scryfall_id.clone()),
                _ => None,
            }),
            1,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to read card prices. {e}"),
                }),
            )
        })?;

        Ok(Json(
            result
                .into_iter()
                .filter_map(|(k, v)| match v {
                    Card::Magic(m) => {
                        let price = price_cache
                            .get(&m.card_identifiers.scryfall_id)
                            .map(api_price_from_cache);
                        Some((k, APICard::from_magic_with_price(m, price)))
                    }
                    _ => None,
                })
                .collect(),
        ))
    }

    async fn get_sets(State(state): State<GathersState>) -> Result<Json<Vec<Set>>, ApiError> {
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
            .map(Json)
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
