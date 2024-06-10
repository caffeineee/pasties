//! `routing::api` responds to requests that should return serialized data to the client. It creates an interface for the `PasteManager` CRUD struct defined in `model`
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

use crate::model::{PasteCreate, PasteDelete, PasteError, PasteManager, PasteReturn};
pub fn routes(manager: PasteManager) -> Router {
    Router::new()
        .route("/new", post(create_paste))
        .route("/:url", get(get_paste_by_url))
        .route("/:url/delete", post(delete_paste_by_url))
        .with_state(manager)
}

async fn create_paste(
    State(manager): State<PasteManager>,
    Json(paste_to_create): Json<PasteCreate>,
) -> Result<(), PasteError> {
    let res = manager.create_paste(paste_to_create).await;
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

async fn delete_paste_by_url(
    State(manager): State<PasteManager>,
    Path(url): Path<String>,
    Json(paste_to_delete): Json<PasteDelete>,
) -> Result<(), PasteError> {
    manager
        .delete_paste_by_url(url, paste_to_delete.password)
        .await
}

pub async fn get_paste_by_url(
    State(manager): State<PasteManager>,
    Path(url): Path<String>,
) -> Result<Json<PasteReturn>, PasteError> {
    let return_paste = manager.get_paste_by_url(url).await;
    match return_paste {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err(e),
    }
}
