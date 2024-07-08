use axum::{routing::get, Router};

use crate::{
    model::PasteManager,
    routing::{api, pages},
};

pub mod database;
pub mod model;
pub mod routing;
pub mod utility;

#[tokio::main]
async fn main() {
    const PORT: u16 = 7878;

    let manager = PasteManager::init().await;

    let app = Router::new()
        .route("/", get(pages::root))
        .merge(pages::routes(manager.clone()))
        .nest("/meta", pages::reserved_routes())
        .nest("/assets", pages::asset_routes())
        .nest("/api", api::routes(manager.clone()))
        .fallback(pages::not_found_handler);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{PORT}"))
        .await
        .unwrap();

    println!("Starting server at http://localhost:{PORT}!");
    axum::serve(listener, app).await.unwrap();
}
