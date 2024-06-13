//! `routing::api` responds to requests that should return serialized data to the client. It creates an interface for the `PasteManager` CRUD struct defined in `model`
use askama_axum::{IntoResponse, Response};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};

use crate::model::{PasteCreate, PasteDelete, PasteError, PasteManager, PasteReturn};
use super::pages;

pub fn routes(manager: PasteManager) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "This is a route reserved for the pasties API.".to_string() })
                .delete(delete_request)
                .post(create_request),
        )
        .route("/:url", get(view_request))
        .fallback(pages::not_found_handler)
        .with_state(manager)
}

async fn create_request(
    State(manager): State<PasteManager>,
    Json(paste_to_create): Json<PasteCreate>,
) -> Result<Response, PasteError> {
    let res = manager.create_paste(paste_to_create).await;
    match res {
        Ok(_) => Ok((StatusCode::CREATED, "Paste created successfully").into_response()),
        Err(e) => Err(e),
    }
}

async fn delete_request(
    State(manager): State<PasteManager>,
    Json(paste_to_delete): Json<PasteDelete>,
) -> Result<Response, PasteError> {
    match manager.delete_paste_by_url(paste_to_delete).await {
        Ok(_) => Ok((StatusCode::OK, "Paste deleted successfully").into_response()),
        Err(e) => Err(e),
    }
}

pub async fn view_request(
    State(manager): State<PasteManager>,
    Path(url): Path<String>,
) -> Result<Json<PasteReturn>, PasteError> {
    match manager.get_paste_by_url(url).await {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err(e),
    }
}
