use std::fs;

use axum::{routing::get, Router};
use pasties::{
    markdown::Markdown, model::PasteManager, routing::{api, pages}
};

#[tokio::main]
async fn main() {
    const PORT: u16 = 7878;

    let md = Markdown::new(fs::read_to_string("test.txt").unwrap());
    println!("{:?}", md.blocks);

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
