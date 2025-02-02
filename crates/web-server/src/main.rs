use axum::Router;
use html_search::router as html_router;
use search_api::set_search_impl;
use search_api_elasticsearch::ElasticsearchRemote;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    //
    // 1) Set the global search backend (Elasticsearch here).
    //
    // This is your custom implementation that implements the `Search` trait
    // from the `search-api` crate.
    set_search_impl(Box::new(ElasticsearchRemote::new("http://localhost:9200")))
        .expect("Failed to set global search backend");

    //
    // 2) Build the main router by merging the `html-search` routes.
    //
    let app = Router::new().merge(html_router());

    //
    // 3) Run the Axum server.
    //
    let port = 3000;
    let addr = format!("0.0.0.0:{}", port);
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
