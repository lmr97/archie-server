// Tests if the hit is received with camel-case JSON, and if the
// correct count is returned.
use tokio;
use reqwest::{self, Client, StatusCode};
mod client_config;



#[tokio::main]
async fn main() {

    let (protocol, mut url) = client_config::get_base_url();
    url.push_str("/hits");
    let req_client = client_config::config_client(protocol);

    let hit_camel_case = String::from(
        "{\"timeStamp\": \"2025-07-07T21:22:00\", \"userAgent\": \"mozilla\"}"
    );

    let hit_count    = test_posting_hit(req_client, url, hit_camel_case).await;

    // demo data starts with 6 hits, the unit tests add 1 more,
    // and the above code adds one more: 6+1+1 == 8
    // 
    // A previous run without TLS, along with the other JS hit,
    // will each add 1 more, so at that point it will be 10.
    // Same goes for the second assert, except it's one more ahead.

    assert!(hit_count == "8" || hit_count == "9");
}

async fn test_posting_hit(req_client: Client, url: String, hit_ser: String) -> String {

    let post_hit_resp = req_client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE,"application/json")
        .body(hit_ser)
        .send()
        .await
        .unwrap();
    assert_eq!(post_hit_resp.status(), StatusCode::OK);

    let get_hits_resp = req_client
        .get(url)
        .send()
        .await
        .unwrap();

    get_hits_resp
        .text()
        .await
        .unwrap()
}