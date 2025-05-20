use axum::{extract::State, response::IntoResponse, routing::post, Json, Router};
use models::filters::CardSearchFilters;
use retrieval::RetrievalSystemTrait;

use crate::GathersState;

pub fn mtg_routes() -> Router<GathersState> {
    async fn search_mtg_cards(
        State(state): State<GathersState>,
        Json(input): Json<CardSearchFilters>,
    ) -> impl IntoResponse {
        // Stuff
        let ret = &state.lock().await.retrieval;
        println!("{:?}", input);
        // TODO: unwraps
        let Some(card) = ret.get_card(input.into()).await.unwrap() else {
            return Json(vec![]);
        };
        Json(vec![card])
    }

    Router::new().route("/cards/search", post(search_mtg_cards))
}
