use async_trait::async_trait;
use elasticsearch::http::transport::{SingleNodeConnectionPool, TransportBuilder};
use elasticsearch::http::Url;
use elasticsearch::{Elasticsearch, SearchParts};
use search_api::{Search, SearchResult, SearchResults};
use serde_json::{json, Value};

#[derive(Default)]
pub struct ElasticsearchStub {}

#[async_trait]
impl Search for ElasticsearchStub {
    async fn search(&self, _query: &str) -> Result<SearchResults, search_api::SearchError> {
        Ok(SearchResults {
            results: vec![SearchResult {
                title: "a stubbed title".to_string(),
                url: "https://notneeded.net/stubbed/blah".to_string(),
                snippets: vec!["a stubbed snippet".to_string()],
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

#[async_trait]
impl Search for ElasticsearchRemote {
    async fn search(&self, query: &str) -> Result<SearchResults, search_api::SearchError> {
        let query = json!({
                  "query": {
                      "query_string": {
                          "query": query.to_string(),
                          "default_field": "body"
                      }
                  },
             "_source": ["title", "url"],
        "highlight": {
          "pre_tags": ["<mark>"],
          "post_tags": ["</mark>"],
          "fields": {
            "body": {
              "fragment_size": 150,
              "number_of_fragments": 3
            }
          }
        }
              });
        println!("query: {:?}", query);

        let response = self
            .client
            .search(SearchParts::Index(&["parks-australia"]))
            .body(query)
            .allow_no_indices(true)
            .send()
            .await
            .unwrap();

        let response_body = response.json::<Value>().await.unwrap();
        println!(
            "response: {}",
            serde_json::to_string_pretty(&response_body).unwrap()
        );

        let results = response_body["hits"]["hits"]
            .as_array()
            .unwrap()
            .iter()
            .map(|hit| SearchResult {
                title: hit["_source"]["title"].as_str().unwrap().to_string(),
                url: hit["_source"]["url"].as_str().unwrap().to_string(),
                snippets: hit["highlight"]["body"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|snippet| snippet.as_str().unwrap().to_string())
                    .collect(),
            })
            .collect();
        Ok(SearchResults { results })
    }
}
