use axum::Router;

use crate::{
    model::Manager,
    routing::{api, pages},
};

pub mod database;
pub mod markdown;
pub mod model;
pub mod routing;
pub mod utility;

#[tokio::main]
async fn main() {
    const PORT: u16 = 7878;

    let manager = Manager::init().await;

    let app = Router::new()
        .merge(pages::routes(manager.clone()))
        .nest("/api", api::routes(manager.clone()))
        .nest("/meta", pages::reserved_routes())
        .nest("/assets", pages::asset_routes())
        .fallback(pages::not_found_handler);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{PORT}"))
        .await
        .unwrap();

    println!("Starting server at http://localhost:{PORT}!");
    axum::serve(listener, app).await.unwrap();
}
