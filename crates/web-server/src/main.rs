use axum::Router;
use clap::Parser;
use html_search::router as html_router;
use search_api::set_search_impl;
use search_api_elasticsearch::ElasticsearchRemote;
use tokio::net::TcpListener;

#[derive(Debug, Clone, Parser)]
struct Cli {
    #[clap(short, long, env, default_value = "http://localhost:9200")]
    elasticsearch_address: String,
    #[clap(short, long, env, default_value = "3000")]
    listen_port: u16,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    set_search_impl(Box::new(ElasticsearchRemote::new(
        &args.elasticsearch_address,
    )))
    .expect("Failed to set global search backend");

    let app = Router::new().merge(html_router());
    let addr = format!("0.0.0.0:{}", args.listen_port);
    println!("Listening on http://{}", addr);

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
