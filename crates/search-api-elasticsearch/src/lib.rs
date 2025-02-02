use search_api::{Search, SearchResult, SearchResults, Snippet};

pub struct ElasticsearchStub {}

impl ElasticsearchStub {
    pub fn new() -> Self {
        ElasticsearchStub {}
    }
}

impl Search for ElasticsearchStub {
    fn search(&self, _query: &str) -> Result<SearchResults, search_api::SearchError> {
        Ok(SearchResults { results: vec![SearchResult{
            title: "a stubbed title".to_string(),
            url: "https://notneeded.net/stubbed/blah".to_string(),
            snippet: Snippet {
                content: "a stubbed snippet".to_string(),
                highlighted: vec![(2, 5)],
            },
        }] })
    }
}
