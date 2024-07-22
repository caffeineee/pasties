//! `routing::api` responds to requests that should return serialized data to the client. It creates an interface for the `PasteManager` CRUD struct defined in `model`
use askama_axum::{IntoResponse, Response};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Form, Json, Router,
};
use serde::Deserialize;

use crate::{
    markdown::render_markdown,
    model::{PasteCreate, PasteDelete, PasteError, PasteManager, PasteReturn, PasteUpdate},
};
use super::pages;

pub struct ApiReturn {
    status:        StatusCode,
    body:          String,
    htmx_redirect: Option<String>,
}

impl IntoResponse for ApiReturn {
    fn into_response(self) -> Response {
        match self.htmx_redirect {
            Some(header) => (self.status, [("HX-Redirect", header)], self.body).into_response(),
            None => (self.status, self.body).into_response(),
        }
    }
}

pub fn routes(manager: PasteManager) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "This is a route reserved for the pasties API.".to_string() })
                .post(create_request)
                .put(update_request)
                .delete(delete_request),
        )
        .route("/:url", get(view_request))
        .route("/render", post(markdown_render_request))
        .fallback(pages::not_found_handler)
        .with_state(manager)
}

async fn create_request(
    State(manager): State<PasteManager>,
    Form(paste_to_create): Form<PasteCreate>,
) -> Result<Response, PasteError> {
    let res = manager.create_paste(paste_to_create).await;
    match res {
        Ok(new_paste) => Ok(ApiReturn {
            status:        StatusCode::CREATED,
            body:          "Paste created successfully".to_string(),
            htmx_redirect: Some(format!("/{}?secret={}", new_paste.url, new_paste.password)),
        }
        .into_response()),
        Err(err) => Err(err),
    }
}

async fn update_request(
    State(manager): State<PasteManager>,
    Form(paste_to_update): Form<PasteUpdate>,
) -> Result<Response, PasteError> {
    match manager.update_paste(paste_to_update).await {
        Ok(updated_paste) => Ok(ApiReturn {
            status:        StatusCode::OK,
            body:          "Paste updated successfully".to_string(),
            htmx_redirect: Some(format!(
                "/{}?updated={}",
                updated_paste.url, updated_paste.password
            )),
        }
        .into_response()),
        Err(e) => Err(e),
    }
}

async fn delete_request(
    State(manager): State<PasteManager>,
    Form(paste_to_delete): Form<PasteDelete>,
) -> Result<Response, PasteError> {
    match manager.delete_paste(paste_to_delete).await {
        Ok(_) => Ok(ApiReturn {
            status:        StatusCode::OK,
            body:          "Paste deleted successfully".to_string(),
            htmx_redirect: None,
        }
        .into_response()),
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

#[derive(Deserialize)]
pub struct StringForm {
    content: String,
}

pub async fn markdown_render_request(Form(markdown): Form<StringForm>) -> Html<String> {
    Html(render_markdown(markdown.content))
}
