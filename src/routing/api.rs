//! `routing::api` responds to requests that should return serialized data to the client. It creates an interface for the `Manager` CRUD struct defined in `model`
use askama_axum::{IntoResponse, Response};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{
    markdown::render_markdown,
    model::{Manager, NewPasteData, PasteCredentials, PasteError, PasteReturn},
};
use super::pages;

pub struct ApiReturn {
    status:        StatusCode,
    body:          String,
    htmx_redirect: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateForm {
    pub url:          String,
    pub password:     String,
    pub content:      String,
    pub new_url:      String,
    pub new_password: String,
}

impl IntoResponse for ApiReturn {
    fn into_response(self) -> Response {
        match self.htmx_redirect {
            Some(header) => (self.status, [("HX-Redirect", header)], self.body).into_response(),
            None => (self.status, self.body).into_response(),
        }
    }
}

pub fn routes(manager: Manager) -> Router {
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
    State(manager): State<Manager>,
    Form(paste_to_create): Form<NewPasteData>,
) -> Result<Response, PasteError> {
    let redirect_url = paste_to_create.url.clone();
    let redirect_secret = paste_to_create.password.clone();
    let res = manager.create_paste(paste_to_create).await;
    match res {
        Ok(()) => Ok(ApiReturn {
            status:        StatusCode::CREATED,
            body:          "Paste created successfully".to_string(),
            htmx_redirect: Some(format!("/{}?secret={}", redirect_url, redirect_secret)),
        }
        .into_response()),
        Err(err) => Err(err),
    }
}

async fn update_request(
    State(manager): State<Manager>,
    Form(paste): Form<UpdateForm>,
) -> Result<Response, PasteError> {
    let credentials = PasteCredentials {
        url:      paste.url,
        password: paste.password,
    };
    let update = NewPasteData {
        url:      paste.new_url,
        password: paste.new_password,
        content:  paste.content,
    };
    let redirect_url = match update.url.is_empty() {
        true => credentials.url.clone(),
        false => update.url.clone(),
    };
    let redirect_secret = update.password.clone();
    match manager.update_paste(credentials, update).await {
        Ok(_) => Ok(ApiReturn {
            status:        StatusCode::OK,
            body:          "Paste updated successfully".to_string(),
            htmx_redirect: Some(format!("/{}?updated={}", redirect_url, redirect_secret)),
        }
        .into_response()),
        Err(e) => Err(e),
    }
}

async fn delete_request(
    State(manager): State<Manager>,
    Form(paste_to_delete): Form<PasteCredentials>,
) -> Result<Response, PasteError> {
    match manager.delete_paste(paste_to_delete).await {
        Ok(_) => Ok(ApiReturn {
            status:        StatusCode::OK,
            body:          "".to_string(),
            htmx_redirect: Some(format!("/?deleted")),
        }
        .into_response()),
        Err(e) => Err(e),
    }
}

pub async fn view_request(
    State(manager): State<Manager>,
    Path(url): Path<String>,
) -> Result<Json<PasteReturn>, PasteError> {
    match manager.retrieve_paste(url).await {
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
