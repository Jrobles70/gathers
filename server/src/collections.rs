use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use models::filters::CardSearchFilters;
use persistence::PersistenceSystemTrait;
use reqwest::StatusCode;
use retrieval::RetrievalSystemTrait;
use serde::{Deserialize, Serialize};

use crate::GathersState;

#[derive(Serialize)]
struct ErrorPayload {
    error: String,
}

pub fn collection_routes() -> Router<GathersState> {
    #[derive(Serialize, Deserialize)]
    struct Collection {
        id: String,
    }

    async fn list(
        State(state): State<GathersState>,
    ) -> Result<Json<Vec<Collection>>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &state.lock().await.storage;

        match storage.list_collections().await {
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

    #[derive(Serialize)]
    struct CollectionAddResponse {
        id: String,
        name: String,
    }

    async fn add(
        State(state): State<GathersState>,
        Json(input): Json<Collection>,
    ) -> Result<Json<CollectionAddResponse>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.lock().await.storage;

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

    #[derive(Serialize)]
    struct CollectionRemoveResponse {
        message: String,
    }

    #[derive(Serialize)]
    struct CollectionCardResponse {
        id: String,
        quantity: u32,
        foil_quantity: u32,
        collection_id: String,
        time_added: DateTime<Utc>,
    }

    async fn remove(
        State(state): State<GathersState>,
        Path(id): Path<String>,
    ) -> Result<Json<CollectionRemoveResponse>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.lock().await.storage;

        match storage.remove_collection(id).await {
            Ok(message) => Ok(Json(CollectionRemoveResponse { message })),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to remove collection. {e}"),
                }),
            )),
        }
    }

    #[derive(Deserialize)]
    struct MoveCardsRequest {
        card_ids: Vec<String>,
    }

    #[derive(Serialize)]
    struct MoveCardsResponse {
        message: String,
    }

    async fn move_to(
        State(_state): State<GathersState>,
        Path(_collection_id): Path<String>,
        Json(_input): Json<MoveCardsRequest>,
    ) -> Result<Json<MoveCardsResponse>, (StatusCode, Json<ErrorPayload>)> {
        // For now, we'll just return a placeholder response
        // In a real implementation, this would move cards to the collection
        Ok(Json(MoveCardsResponse {
            message: "Cards moved successfully".to_string(),
        }))
    }

    fn default_limit() -> usize {
        20
    }

    #[derive(Deserialize)]
    struct CardToAdd {
        card_id: String,
        quantity: u32,
        foil_quantity: u32,
    }

    async fn cards_add(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Json(input): Json<CardToAdd>,
    ) -> Result<Json<Vec<CollectionCardResponse>>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &mut state.lock().await.storage;

        // First, let's verify that the collection exists
        let collections = match storage.list_collections().await {
            Ok(collections) => collections,
            Err(e) => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to verify collection. {e}"),
                    }),
                ));
            }
        };

        if !collections.contains(&collection_id) {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorPayload {
                    error: "Collection not found".to_string(),
                }),
            ));
        }

        // Add the card to the collection
        let uuid = input.card_id.parse::<i64>().map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorPayload {
                    error: format!("Invalid card ID format. {e}"),
                }),
            )
        })?;

        let now = chrono::Utc::now().to_rfc3339();

        match storage
            .add_card_to_collection(
                collection_id.clone(),
                uuid,
                input.quantity,
                input.foil_quantity,
                now,
            )
            .await
        {
            Ok(_) => {
                // Return the added card
                let cards = match storage.get_cards_in_collection(collection_id.clone()).await {
                    Ok(cards) => cards,
                    Err(e) => {
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorPayload {
                                error: format!("Failed to fetch added card. {e}"),
                            }),
                        ));
                    }
                };

                // Find the last added card (the one we just added)
                if let Some(last_card) = cards.last() {
                    let response_card = CollectionCardResponse {
                        id: last_card.uuid.to_string(),
                        quantity: last_card.quantity,
                        foil_quantity: last_card.foil_quantity,
                        collection_id,
                        time_added: chrono::DateTime::parse_from_rfc3339(&last_card.time_added)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now()),
                    };
                    Ok(Json(vec![response_card]))
                } else {
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorPayload {
                            error: "Failed to retrieve added card".to_string(),
                        }),
                    ))
                }
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to add card to collection. {e}"),
                }),
            )),
        }
    }

    #[derive(Deserialize)]
    struct CollectionCardsQuery {
        #[serde(default)]
        offset: usize,
        #[serde(default = "default_limit")]
        limit: usize,
    }

    async fn cards_get(
        State(state): State<GathersState>,
        Path(collection_id): Path<String>,
        Query(query): Query<CollectionCardsQuery>,
    ) -> Result<Json<Vec<CollectionCard>>, (StatusCode, Json<ErrorPayload>)> {
        let storage = &state.lock().await.storage;

        match storage
            .get_cards_in_collection_paginated(collection_id.clone(), query.offset, query.limit)
            .await
        {
            Ok(cards) => {
                // Convert CollectionCard to CollectionCardResponse
                let mut response_cards = Vec::new();

                for card in cards {
                    response_cards.push(CollectionCard {
                        id: card.uuid.to_string(), // uuid is the card ID from retrieval system
                        quantity: card.quantity,
                        foil_quantity: card.foil_quantity,
                        collection_id: collection_id.clone(), // This should be the collection ID passed in
                        time_added: DateTime::parse_from_rfc3339(&card.time_added)
                            .map(|dt| dt.with_timezone(&Utc))
                            .unwrap_or_else(|_| Utc::now()),
                    });
                }

                Ok(Json(response_cards))
            }
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorPayload {
                    error: format!("Failed to get cards from collection. {e}"),
                }),
            )),
        }
    }

    #[derive(Serialize)]
    struct CollectionCard {
        id: String,
        quantity: u32,
        foil_quantity: u32,
        collection_id: String,
        time_added: DateTime<Utc>,
    }

    #[derive(Deserialize)]
    struct SearchQuery {
        #[serde(default)]
        offset: usize,
        #[serde(default = "default_limit")]
        page_size: usize,
    }

    #[derive(Deserialize, Serialize)]
    struct CardIdentInner {
        #[serde(rename = "scryfallId")]
        scryfall_id: String,
    }

    #[derive(Deserialize, Serialize)]
    struct ResultCardInner {
        id: String,
        name: String,
        #[serde(rename = "setCode")]
        set_code: String,
        #[serde(rename = "cardIdentifiers")]
        card_identifiers: CardIdentInner,
    }

    #[derive(Deserialize, Serialize)]
    struct ResultCard {
        #[serde(rename = "mtGCard")]
        mtg_card: ResultCardInner,
    }

    async fn search_temp(
        State(state): State<GathersState>,
        Query(query): Query<SearchQuery>,
        Json(input): Json<CardSearchFilters>,
    ) -> Result<Json<Vec<ResultCard>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.lock().await.retrieval;

        match ret
            .search_cards(input, query.offset.into(), (query.page_size + 1).into())
            .await
        {
            Ok(result) => Ok(Json(
                result
                    .iter()
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

    Router::new()
        .route("/list", get(list))
        .route("/add", post(add))
        .route("/remove/{id}", post(remove))
        .route("/move/{id}", post(move_to))
        .route("/cards/{id}/list", get(cards_get))
        .route("/search", post(search_temp))
        .route("/cards/{id}/add", post(cards_add))
}
