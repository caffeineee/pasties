use axum::{routing::get, Router};
use pasties::{model::PasteManager, routing::{api, pages}};

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
#[cfg(test)]
mod tests {
    use pasties::database;

    #[test]
    fn does_db_work() {
        assert_eq!("random contend idek what to put here".to_string(), database::Database::init().get_paste_by_url(&"newpaste".to_string()).unwrap().content)
    }

    // #[test]
    // fn manual_db() {
    //     let c = database::Database::init().connection;
    //     let s = c.prepare("select * from pastes where url='hello';").unwrap();
    //     println!("{:?}", s.query(()).unwrap().map(|a|{a}).into())
    // }
}