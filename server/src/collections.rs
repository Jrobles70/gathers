use std::collections::HashMap;

use aide::axum::{
    ApiRouter,
    routing::{get, post},
};
use axum::extract::Multipart;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::{DateTime, Utc};
use models::Card;
use persistence::{CollectionCardsParams, PersistenceSystem, PersistenceSystemTrait};
use retrieval::{NamedRetrievalSystem as _, RetrievalSystem, RetrievalSystemTrait};

use crate::{
    ApiError, ErrorPayload, GathersState,
    collections::collections_models::{
        APICardSearchFilters, CardIdentInner, CardToAdd, CollectionAddResponse, CollectionCard,
        CollectionCardsQuery, CollectionRemoveResponse, CollectionsSearchQuery, ResultCard,
        ResultCardInner,
    },
};
pub mod collections_models;

use crate::collections::collections_models::Collection;

/// Returns all configured retrieval systems, cloned out of the state lock,
/// keyed by their provider name.
async fn clone_retrieval_systems_by_name(state: &GathersState) -> HashMap<String, RetrievalSystem> {
    let guard = state.0.lock().await;
    [
        guard.mtg.clone(),
        guard.riftbound.clone(),
        guard.pokemon.clone(),
    ]
    .into_iter()
    .flatten()
    .map(|s| (s.name().to_string(), s))
    .collect()
}

pub fn collection_routes() -> ApiRouter<GathersState> {
    async fn list(State(state): State<GathersState>) -> Result<Json<Vec<Collection>>, ApiError> {
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
    ) -> Result<Json<CollectionAddResponse>, ApiError> {
        validate_collection_name(&input.id)?;

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
    ) -> Result<Json<CollectionRemoveResponse>, ApiError> {
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
    ) -> Result<Json<()>, ApiError> {
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

    fn validate_collection_name(name: &str) -> Result<(), ApiError> {
        if name.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Collection name cannot be empty".to_string(),
                }),
            ));
        }
        if name.len() > 255 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Collection name too long (max 255 characters)".to_string(),
                }),
            ));
        }
        if name.chars().any(|c| c.is_control()) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Collection name contains invalid characters".to_string(),
                }),
            ));
        }
        Ok(())
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
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
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
                provider: card.provider,
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
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        // Identify the provider by finding which configured system has this card.
        let systems = clone_retrieval_systems_by_name(&state).await;
        let card_ids = vec![input.id.clone()];
        let mut provider = String::new();
        for (name, system) in &systems {
            if let Ok(found) = system.get_cards_by_ids(card_ids.clone()).await
                && !found.is_empty()
            {
                provider = name.clone();
                break;
            }
        }

        let storage = &mut state.1.lock().await.storage;

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
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        let storage = &mut state.1.lock().await.storage;

        if let Err(e) = validate_collection(storage, &collection_id).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, e));
        };

        let neg_quantity = input.quantity.checked_neg().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Invalid quantity".to_string(),
                }),
            )
        })?;
        let neg_foil_quantity = input.foil_quantity.checked_neg().ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "Invalid foil quantity".to_string(),
                }),
            )
        })?;

        mutate_card_quantities(
            storage,
            &collection_id,
            input.id,
            neg_quantity,
            neg_foil_quantity,
            "".to_string(),
        )
        .await
    }

    async fn cards_get(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Query(query): Query<CollectionCardsQuery>,
    ) -> Result<Json<Vec<CollectionCard>>, ApiError> {
        let collection_params = CollectionCardsParams {
            offset: query.offset,
            limit: query.limit.min(1000),
            sort_by: query.sort_by.map(persistence::CollectionSortField::from),
            sort_order: query.sort_order.map(models::filters::SortOrder::from),
            provider: query.provider,
        };
        let cards = state
            .1
            .lock()
            .await
            .storage
            .get_cards_in_collection_paginated(&collection_id, collection_params)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to get cards from collection. {e}"),
                    }),
                )
            })?;

        let response_cards = cards
            .into_iter()
            .map(|card| CollectionCard {
                id: card.uuid,
                quantity: card.quantity,
                foil_quantity: card.foil_quantity,
                collection_id: collection_id.clone(),
                time_added: DateTime::parse_from_rfc3339(&card.time_added)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                provider: card.provider,
            })
            .collect();

        Ok(Json(response_cards))
    }

    async fn collection_cards_count(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
    ) -> Result<Json<usize>, ApiError> {
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
    ) -> Result<Json<Vec<ResultCard>>, ApiError> {
        let guard = state.0.lock().await;
        let ret = guard.require_mtg()?;

        match ret
            .search_cards(input.into(), query.offset.into(), query.page_size.min(1000).into())
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

    async fn export(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
    ) -> Result<Response, ApiError> {
        let retrievals: Vec<RetrievalSystem> = {
            let guard = state.0.lock().await;
            [guard.mtg.clone(), guard.riftbound.clone(), guard.pokemon.clone()]
                .into_iter()
                .flatten()
                .collect()
        };

        let csv = state
            .1
            .lock()
            .await
            .storage
            .export_collection(&collection_id, &retrievals)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Export failed: {e}"),
                    }),
                )
            })?;

        let safe_filename: String = collection_id
            .chars()
            .filter(|c| *c != '"' && *c != '\\' && !c.is_control())
            .collect();
        let filename = format!("attachment; filename=\"{safe_filename}.csv\"");
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"))
            .header(
                header::CONTENT_DISPOSITION,
                HeaderValue::from_str(&filename).unwrap_or_else(|_| {
                    HeaderValue::from_static("attachment; filename=\"collection.csv\"")
                }),
            )
            .body(axum::body::Body::from(csv))
            .unwrap())
    }

    async fn import(
        State(state): State<GathersState>,
        mut multipart: Multipart,
    ) -> Result<Json<()>, ApiError> {
        let mut file_bytes: Option<Vec<u8>> = None;
        let mut collection_name: Option<String> = None;

        while let Some(field) = multipart.next_field().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: format!("Failed to read multipart field: {e}"),
                }),
            )
        })? {
            match field.name() {
                Some("file") => {
                    file_bytes = Some(field.bytes().await.map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorPayload {
                                error: format!("Failed to read file bytes: {e}"),
                            }),
                        )
                    })?.to_vec());
                }
                Some("collection") => {
                    collection_name = Some(field.text().await.map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorPayload {
                                error: format!("Failed to read collection field: {e}"),
                            }),
                        )
                    })?);
                }
                _ => {}
            }
        }

        let bytes = file_bytes.ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: "No file provided".to_string(),
                }),
            )
        })?;

        let collection_name = collection_name.unwrap_or_else(|| "New Collection".to_string());
        validate_collection_name(&collection_name)?;

        let mut tmp = tempfile::NamedTempFile::new().map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to create temp file: {e}"),
                }),
            )
        })?;
        std::io::Write::write_all(&mut tmp, &bytes).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to write temp file: {e}"),
                }),
            )
        })?;
        let tmp_path = tmp.path().to_string_lossy().to_string();

        let retrievals: Vec<RetrievalSystem> = {
            let guard = state.0.lock().await;
            [guard.mtg.clone(), guard.riftbound.clone(), guard.pokemon.clone()]
                .into_iter()
                .flatten()
                .collect()
        };

        state
            .1
            .lock()
            .await
            .storage
            .import_csv(tmp_path, collection_name, &retrievals, None)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Import failed: {e}"),
                    }),
                )
            })?;

        Ok(Json(()))
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
        .route("/import", axum::routing::post(import))
        .route("/export/{id}", axum::routing::get(export))
}
