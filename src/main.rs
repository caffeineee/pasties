use axum::{routing::get, Router};
use pasties::{model::PasteManager, routing::api, routing::pages};

#[tokio::main]
async fn main() {
    const PORT: u16 = 7878;

    let manager = PasteManager::init().await;

    let app = Router::new()
        .route("/", get(pages::root))
        .merge(pages::routes(manager.clone()))
        .nest("/api", api::routes(manager.clone()))
        .fallback(pages::not_found_handler);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{PORT}"))
        .await
        .unwrap();

    println!("Starting server at http://localhost:{PORT}!");
    axum::serve(listener, app).await.unwrap();
}
