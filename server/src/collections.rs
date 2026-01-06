use axum::{
    extract::State,
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
    async fn list(
        State(_state): State<GathersState>,
    ) -> Result<Json<Vec<String>>, (StatusCode, Json<ErrorPayload>)> {
        // Return an empty JSON array for now
        Ok(Json(Vec::<String>::new()))
    }

    async fn add(
        State(state): State<GathersState>,
    ) -> Result<Json<String>, (StatusCode, Json<ErrorPayload>)> {
        let mut s = state.lock().await.storage.clone();
        s.add_collection("Some Collection".to_string())
            .await
            .map(|r| Json(r))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorPayload {
                        error: format!("Failed to search cards. {e}"),
                    }),
                )
            })
    }

    async fn remove(
        State(state): State<GathersState>,
    ) -> Result<Json<String>, (StatusCode, Json<ErrorPayload>)> {
        return Ok(Json("Heya".to_string()));
    }

    async fn move_to(
        State(state): State<GathersState>,
    ) -> Result<Json<String>, (StatusCode, Json<ErrorPayload>)> {
        return Ok(Json("Heya".to_string()));
    }

    async fn cards_get(
        State(state): State<GathersState>,
    ) -> Result<Json<String>, (StatusCode, Json<ErrorPayload>)> {
        return Ok(Json("Heya".to_string()));
    }

    fn default_limit() -> usize {
        1
    }

    #[derive(Deserialize)]
    struct SearchQuery {
        #[serde(default)]
        offset: usize,
        #[serde(default = "default_limit")]
        pageSize: usize,
    }

    #[derive(Deserialize, Serialize)]
    struct CardIdentInner {
        scryfallId: String,
    }

    #[derive(Deserialize, Serialize)]
    struct ResultCardInner {
        id: String,
        name: String,
        setCode: String,
        cardIdentifiers: CardIdentInner,
    }

    #[derive(Deserialize, Serialize)]
    struct ResultCard {
        mtGCard: ResultCardInner,
    }

    async fn search_temp(
        State(state): State<GathersState>,
        Query(query): Query<SearchQuery>,
        Json(input): Json<CardSearchFilters>,
    ) -> Result<Json<Vec<ResultCard>>, (StatusCode, Json<ErrorPayload>)> {
        let ret = &state.lock().await.retrieval;

        match ret
            .search_cards(input, query.offset.into(), (query.pageSize + 1).into())
            .await
        {
            Ok(result) => Ok(Json(
                result
                    .iter()
                    .map(|c| ResultCard {
                        mtGCard: ResultCardInner {
                            id: c.id.clone(),
                            name: c.name.clone(),
                            setCode: c.set_code.clone(),
                            cardIdentifiers: CardIdentInner {
                                scryfallId: c.card_identifiers.scryfall_id.clone(),
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
