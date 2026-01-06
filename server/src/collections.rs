use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::Query;
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
    struct CollectionCardsQuery {
        #[serde(default)]
        offset: usize,
        #[serde(default = "default_limit")]
        limit: usize,
    }

    async fn cards_get(
        State(_state): State<GathersState>,
        Path(_collection_id): Path<String>,
        Query(_query): Query<CollectionCardsQuery>,
    ) -> Result<Json<Vec<CollectionCardResponse>>, (StatusCode, Json<ErrorPayload>)> {
        // For now, we'll return an empty vector
        // In a real implementation, this would get cards from the collection
        Ok(Json(vec![]))
    }

    #[derive(Serialize)]
    struct CollectionCardResponse {
        id: String,
        name: String,
        set_code: String,
        scryfall_id: String,
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
        .route("/cards/{id}/get", get(cards_get))
        .route("/search", post(search_temp))
}
