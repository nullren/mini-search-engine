use async_trait::async_trait;
use once_cell::sync::OnceCell;
use std::error::Error as StdError;

#[derive(Debug)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: Snippet,
}

#[derive(Debug)]
pub struct Snippet {
    pub content: String,
    pub highlighted: Vec<(usize, usize)>,
}

#[derive(Debug)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
}

#[derive(Debug)]
pub enum SearchError {
    Internal(String),
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::Internal(msg) => write!(f, "Search error: {}", msg),
        }
    }
}

impl From<Box<dyn StdError>> for SearchError {
    fn from(err: Box<dyn StdError>) -> Self {
        SearchError::Internal(err.to_string())
    }
}

impl StdError for SearchError {}

#[async_trait]
pub trait Search: Send + Sync {
    async fn search(&self, query: &str) -> Result<SearchResults, SearchError>;
}

static GLOBAL_SEARCH_IMPL: OnceCell<Box<dyn Search>> = OnceCell::new();

pub fn set_search_impl(backend: Box<dyn Search>) -> Result<(), Box<dyn StdError>> {
    GLOBAL_SEARCH_IMPL
        .set(backend)
        .map_err(|_| "Search implementation is already set!".into())
}

pub async fn search(query: &str) -> Result<SearchResults, SearchError> {
    if let Some(backend) = GLOBAL_SEARCH_IMPL.get() {
        backend.search(query).await
    } else {
        Err(SearchError::Internal(
            "No search implementation set!".to_string(),
        ))
    }
}
