use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use models::Card;
use persistence::{PersistenceSystem, PersistenceSystemTrait};
use reqwest::StatusCode;
use retrieval::{NamedRetrievalSystem as _, RetrievalSystemTrait};
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    GathersState,
    collections::collections_models::{
        APICardSearchFilters, CardIdentInner, CardToAdd, CollectionAddResponse, CollectionCard,
        CollectionCardsQuery, CollectionRemoveResponse, CollectionsSearchQuery, ResultCard,
        ResultCardInner,
    },
};
pub mod collections_models;

use crate::collections::collections_models::Collection;

#[derive(Serialize, JsonSchema)]
struct ErrorPayload {
    error: String,
}

pub fn collection_routes() -> ApiRouter<GathersState> {
    async fn list(
        State(state): State<GathersState>,
    ) -> Result<Json<Vec<Collection>>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &state.1.lock().await.storage;

        match storage.list_collections(None).await {
            Ok(collections) => Ok(Json(
                collections
                    .iter()
                    .map(|c| Collection { id: c.clone() })
                    .collect(),
            )),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to list collections. {e}"),
                }),
            )),
        }
    }

    async fn add(
        State(state): State<GathersState>,
        Json(input): Json<Collection>,
    ) -> Result<Json<CollectionAddResponse>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.1.lock().await.storage;

        match storage.add_collection(input.id.clone()).await {
            Ok(collection_id) => Ok(Json(CollectionAddResponse {
                id: collection_id,
                name: input.id,
            })),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to add collection. {e}"),
                }),
            )),
        }
    }

    async fn remove(
        State(state): State<GathersState>,
        Path(id): Path<String>,
    ) -> Result<Json<CollectionRemoveResponse>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.1.lock().await.storage;

        // TODO: allow setting the "move to collection" instead of None
        match storage.remove_collection(&id, None).await {
            Ok(message) => Ok(Json(CollectionRemoveResponse { message })),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to remove collection. {e}"),
                }),
            )),
        }
    }

    #[axum::debug_handler]
    async fn move_to(
        State(state): State<GathersState>,
        Path(to_collection_id): Path<String>,
        Json(input): Json<Vec<CollectionCard>>,
    ) -> Result<Json<()>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.1.lock().await.storage;

        let cards: Vec<models::CollectionCard> = input.iter().map(|card| card.into()).collect();
        match storage
            .move_cards_between_collections(&cards, to_collection_id)
            .await
        {
            Ok(_) => Ok(Json(())),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to move cards. {e}"),
                }),
            )),
        }
    }

    async fn validate_collection(
        storage: &mut PersistenceSystem,
        collection_id: &String,
    ) -> Result<(), Json<ErrorPayload>> {
        let collections = match storage.list_collections(None).await {
            Ok(collections) => collections,
            Err(e) => {
                return Err(Json(ErrorPayload {
                    error: format!("Failed to verify collection. {e}"),
                }));
            }
        };

        if !collections.contains(collection_id) {
            return Err(Json(ErrorPayload {
                error: "Collection not found".to_string(),
            }));
        }
        Ok(())
    }

    async fn mutate_card_quantities(
        storage: &mut PersistenceSystem,
        collection_id: &str,
        uuid: String,
        quantity: i32,
        foil_quantity: i32,
        provider: String,
    ) -> Result<Json<Vec<CollectionCard>>, (StatusCode, Json<ErrorPayload>)> {
        let now = chrono::Utc::now();
        let now_str = now.to_rfc3339();

        match storage
            .add_card_to_collection(
                &collection_id.to_string(),
                &uuid,
                quantity,
                foil_quantity,
                &now_str,
                &provider,
            )
            .await
        {
            Ok(card) => Ok(Json(vec![CollectionCard {
                id: card.uuid.to_string(),
                quantity: card.quantity,
                foil_quantity: card.foil_quantity,
                collection_id: collection_id.to_string(),
                time_added: DateTime::parse_from_rfc3339(&card.time_added)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }])),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to change card quantity in collection. {e}"),
                }),
            )),
        }
    }

    async fn cards_add(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Json(input): Json<CardToAdd>,
    ) -> Result<Json<Vec<CollectionCard>>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.1.lock().await.storage;
        let provider = state.0.lock().await.retrieval.name().to_string();

        if let Err(e) = validate_collection(storage, &collection_id).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e));
        };

        mutate_card_quantities(
            storage,
            &collection_id,
            input.id,
            input.quantity,
            input.foil_quantity,
            provider,
        )
        .await
    }

    async fn cards_remove(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Json(input): Json<CardToAdd>,
    ) -> Result<Json<Vec<CollectionCard>>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.1.lock().await.storage;

        if let Err(e) = validate_collection(storage, &collection_id).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e));
        };

        mutate_card_quantities(
            storage,
            &collection_id,
            input.id,
            -input.quantity,
            -input.foil_quantity,
            "".to_string(),
        )
        .await
    }

    async fn cards_get(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Query(query): Query<CollectionCardsQuery>,
    ) -> Result<Json<Vec<CollectionCard>>, (StatusCode, Json<ErrorPayload>)> {
        let raw_cards = state
            .1
            .lock()
            .await
            .storage
            .get_cards_in_collection_paginated(&collection_id, query.offset, query.limit)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to get cards from collection. {e}"),
                    }),
                )
            })?;

        if raw_cards.is_empty() {
            return Ok(Json(vec![]));
        }

        let ids: Vec<String> = raw_cards.iter().map(|c| c.uuid.clone()).collect();
        let found = state
            .0
            .lock()
            .await
            .retrieval
            .get_cards_by_ids(ids)
            .await
            .unwrap_or_default();

        let response_cards = raw_cards
            .into_iter()
            .filter(|card| found.contains_key(&card.uuid))
            .map(|card| CollectionCard {
                id: card.uuid,
                quantity: card.quantity,
                foil_quantity: card.foil_quantity,
                collection_id: collection_id.clone(),
                time_added: DateTime::parse_from_rfc3339(&card.time_added)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
            .collect();

        Ok(Json(response_cards))
    }

    async fn collection_cards_count(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
    ) -> Result<Json<usize>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.1.lock().await.storage;

        match storage.get_cards_in_collection_count(collection_id).await {
            Ok(count) => Ok(Json(count)),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to get card count for collection. {e}"),
                }),
            )),
        }
    }

    async fn search_temp(
        State(state): State<GathersState>,
        Query(query): Query<CollectionsSearchQuery>,
        Json(input): Json<APICardSearchFilters>,
    ) -> Result<Json<Vec<ResultCard>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.0.lock().await.retrieval;

        match ret
            .search_cards(input.into(), query.offset.into(), query.page_size.into())
            .await
        {
            Ok(result) => Ok(Json(
                result
                    .iter()
                    .filter_map(|c| match c {
                        Card::Magic(m) => Some(m),
                        _ => None,
                    })
                    .map(|c| ResultCard {
                        mtg_card: ResultCardInner {
                            id: c.id.clone(),
                            name: c.name.clone(),
                            set_code: c.set_code.clone(),
                            card_identifiers: CardIdentInner {
                                scryfall_id: c.card_identifiers.scryfall_id.clone(),
                            },
                        },
                    })
                    .collect(),
            )),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to search cards. {e}"),
                }),
            )),
        }
    }

    ApiRouter::new()
        .api_route("/list", get(list))
        .api_route("/add", post(add))
        .api_route("/remove/{id}", post(remove))
        .api_route("/move/{id}", post(move_to))
        .api_route("/cards/{id}/list", get(cards_get))
        .api_route("/cards/{id}/count", get(collection_cards_count))
        .api_route("/search", post(search_temp))
        .api_route("/cards/{id}/add", post(cards_add))
        .api_route("/cards/{id}/delete", post(cards_remove))
}
