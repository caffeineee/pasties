//! `routing::pages` responds to requests that should return rendered HTML (or other website assets) to the client
use std::fs;

use askama_axum::Template;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;

use crate::{
    markdown::render_markdown,
    model::{Manager, PasteReturn},
};

pub fn routes(manager: Manager) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/:url", get(view_paste_by_url))
        .route("/:url/edit", get(edit_paste_by_url))
        .with_state(manager)
}

pub fn reserved_routes() -> Router {
    Router::new().route(
        "/",
        get(|| async { "This is a route reserved for pasties.".to_string() }),
    )
}

pub fn asset_routes() -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { "This is a route reserved for pasties assets.".to_string() }),
        )
        .route(
            "/style.css",
            get(|| async {
                let stylesheet = fs::read_to_string("./assets/style.css");
                match stylesheet {
                    Err(e) => panic!("FATAL: Reading the stylesheet failed: {e}"),
                    Ok(s) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/css")], s),
                }
            }),
        )
}

#[derive(Deserialize, Debug, Clone)]
struct Modal {
    secret:  Option<String>,
    updated: Option<String>,
    message: Option<String>,
}

#[derive(Template)]
#[template(path = "paste.html")]
struct PasteView {
    title: String,
    paste: PasteReturn,
    modal: Modal,
}

#[derive(Template)]
#[template(path = "editor.html")]
struct EditorView {
    title: String,
    paste: Option<PasteReturn>,
}

#[derive(Template)]
#[template(path = "infoview.html")]
struct InfoView {
    title:   String,
    content: String,
}

pub async fn root() -> impl IntoResponse {
    let editor = EditorView {
        title: "".to_string(),
        paste: None,
    };
    Html(editor.render().unwrap())
}

async fn edit_paste_by_url(
    Path(url): Path<String>,
    State(manager): State<Manager>,
) -> impl IntoResponse {
    match manager.retrieve_paste(url).await {
        Ok(paste) => Html(
            EditorView {
                title: paste.url.clone(),
                paste: Some(paste),
            }
            .render()
            .unwrap(),
        ),
        Err(e) => Html(
            InfoView {
                title:   "Error".to_string(),
                content: e.to_string(),
            }
            .render()
            .unwrap(),
        ),
    }
}

async fn view_paste_by_url(
    Path(url): Path<String>,
    Query(modal_query): Query<Modal>,
    State(manager): State<Manager>,
) -> impl IntoResponse {
    match manager.retrieve_paste(url).await {
        Ok(mut paste) => {
            paste.content = render_markdown(paste.content);
            let paste_render = PasteView {
                title: paste.url.to_string(),
                paste,
                modal: modal_query,
            };
            Html(paste_render.render().unwrap())
        }
        Err(e) => {
            let paste_render = InfoView {
                title:   "Error".to_string(),
                content: e.to_string(),
            };
            Html(paste_render.render().unwrap())
        }
    }
}

pub async fn not_found_handler() -> impl IntoResponse {
    Html(
        InfoView {
            title:   "Error 404".to_string(),
            content: "The requested resource could not be found".to_string(),
        }
        .render()
        .unwrap(),
    )
}
