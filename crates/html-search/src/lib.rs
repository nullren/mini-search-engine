use askama::Template;
use axum::response::Html;
use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use search_api::{search, SearchResults};
use std::collections::HashMap;

pub struct SearchResultView {
    pub title: String,
    pub url: String,
    /// Already contains `<mark>` tags, etc.
    /// We'll mark this as "safe" in the template.
    pub snippets: Vec<String>,
}

pub struct SearchResultsView {
    pub results: Vec<SearchResultView>,
}

pub fn to_search_results_view(sr: &SearchResults) -> SearchResultsView {
    let results = sr
        .results
        .iter()
        .map(|r| SearchResultView {
            title: r.title.clone(),
            url: r.url.clone(),
            snippets: r.snippets.to_vec(),
        })
        .collect();
    SearchResultsView { results }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "search_results.html")]
pub struct SearchResultsTemplate {
    pub data: SearchResultsView,
}

/// Show a simple HTML form that points to `/search`.
async fn get_index() -> impl IntoResponse {
    // Renders the index.html template (see templates below)
    Html(
        IndexTemplate
            .render()
            .unwrap_or_else(|_| "Template error".into()),
    )
}

/// Parse `?q=...`, call the global search, and display results.
async fn get_search(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    let query = params.get("q").cloned().unwrap_or_default();

    let search_res: Result<SearchResults, _> = search(&query).await;

    match search_res {
        Ok(result) => {
            let view = to_search_results_view(&result);
            let tpl = SearchResultsTemplate { data: view };
            Html(tpl.render().unwrap_or_else(|_| "Template error".into()))
        }
        Err(err) => Html(format!("Search failed: {err}")),
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_index))
        .route("/search", get(get_search))
}
