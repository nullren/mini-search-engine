use askama::Template;
use axum::response::Html;
use axum::{extract::Query, response::IntoResponse, routing::get, Router};
use search_api::{search, SearchResults, Snippet};
use std::collections::HashMap;

/// Convert the snippet into an HTML string with highlight markup.
/// e.g. given "hello world" with highlight [(0,5)], returns "<mark>hello</mark> world"
pub fn snippet_to_html(snippet: &Snippet) -> String {
    let mut result = String::new();
    let mut prev_end = 0;
    let content = snippet.content.as_str();

    // Sort ranges just in case
    let mut ranges = snippet.highlighted.clone();
    ranges.sort_by_key(|r| r.0);

    for (start, end) in ranges {
        // Push everything before this highlight
        if start > prev_end {
            // add unmarked text
            result.push_str(&content[prev_end..start]);
        }
        // Add the highlighted text
        result.push_str("<mark>");
        result.push_str(&content[start..end]);
        result.push_str("</mark>");

        prev_end = end;
    }

    // If there's leftover text after the last highlight, add it
    if prev_end < content.len() {
        result.push_str(&content[prev_end..]);
    }

    result
}

pub struct SearchResultView {
    pub title: String,
    pub url: String,
    /// Already contains `<mark>` tags, etc.
    /// We'll mark this as "safe" in the template.
    pub snippet_html: String,
}

pub struct SearchResultsView {
    pub results: Vec<SearchResultView>,
}

/// Convert a `SearchResults` object into a `SearchResultsView`
/// for easy rendering in the template.
pub fn to_search_results_view(sr: &SearchResults) -> SearchResultsView {
    let results = sr
        .results
        .iter()
        .map(|r| SearchResultView {
            title: r.title.clone(),
            url: r.url.clone(),
            snippet_html: snippet_to_html(&r.snippet),
        })
        .collect();

    SearchResultsView { results }
}

//
// 1) Define Askama templates
//

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

#[derive(Template)]
#[template(path = "search_results.html")]
pub struct SearchResultsTemplate {
    pub data: SearchResultsView,
}

//
// 2) Define handlers
//

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

    // Call the global search (which must be set by the main crate)
    let search_res: Result<SearchResults, _> = search(&query).await;

    match search_res {
        Ok(result) => {
            // Transform for rendering (adding highlights to HTML)
            let view = to_search_results_view(&result);

            let tpl = SearchResultsTemplate { data: view };
            Html(tpl.render().unwrap_or_else(|_| "Template error".into()))
        }
        Err(err) => Html(format!("Search failed: {err}")),
    }
}

//
// 3) Build the router for these endpoints
//
pub fn router() -> Router {
    Router::new()
        .route("/", get(get_index))
        .route("/search", get(get_search))
}
