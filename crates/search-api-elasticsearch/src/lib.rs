use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::http::Url;
use elasticsearch::Elasticsearch;
use search_api::{Search, SearchResult, SearchResults, Snippet};
use serde_json::json;

pub struct ElasticsearchStub {}

impl ElasticsearchStub {
    pub fn new() -> Self {
        ElasticsearchStub {}
    }
}

impl Search for ElasticsearchStub {
    fn search(&self, _query: &str) -> Result<SearchResults, search_api::SearchError> {
        Ok(SearchResults {
            results: vec![SearchResult {
                title: "a stubbed title".to_string(),
                url: "https://notneeded.net/stubbed/blah".to_string(),
                snippet: Snippet {
                    content: "a stubbed snippet".to_string(),
                    highlighted: vec![(2, 5)],
                },
            }],
        })
    }
}

pub struct ElasticsearchRemote {
    client: Elasticsearch,
}

impl ElasticsearchRemote {
    pub fn new(addr: &str) -> Self {
        let url = Url::parse(addr).unwrap();
        // there is a round-robin connection pool available as well
        let conn_pool = SingleNodeConnectionPool::new(url);
        let transport = TransportBuilder::new(conn_pool)
            .disable_proxy()
            .build()
            .unwrap();
        let client = Elasticsearch::new(transport);
        ElasticsearchRemote { client }
    }
}

impl Search for ElasticsearchRemote {
    fn search(&self, query: &str) -> Result<SearchResults, search_api::SearchError> {
        let response = self
            .client
            .search()
            .body(json!({
                "query": {
                    "match": {
                        "content": query
                    }
                }
            }))
            .send()
            .await
            .unwrap();
        let hits = response.json::<serde_json::Value>().unwrap()["hits"]["hits"]
            .as_array()
            .unwrap();
        let results = hits
            .iter()
            .map(|hit| SearchResult {
                title: hit["_source"]["title"].as_str().unwrap().to_string(),
                url: hit["_source"]["url"].as_str().unwrap().to_string(),
                snippet: Snippet {
                    content: hit["_source"]["content"].as_str().unwrap().to_string(),
                    highlighted: vec![],
                },
            })
            .collect();
        Ok(SearchResults { results })
    }
}
