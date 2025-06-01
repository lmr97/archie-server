use tokio;
use reqwest::{self, StatusCode};
use mysql_common::serde_json;
use custom_backend::types::db_io_types::WebpageHit;
mod client_config;


#[tokio::main]
async fn main() {

    let (protocol, mut url) = client_config::get_base_url();
    url.push_str("/hits");
    let req_client = client_config::config_client(protocol);

    let new_hit = WebpageHit::default();

    let post_hit_resp = req_client
        .post(&url)
        .header(reqwest::header::CONTENT_TYPE,"application/json")
        .body(serde_json::to_string(&new_hit).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(post_hit_resp.status(), StatusCode::OK);

    let get_hits_resp = req_client
        .get(url)
        .send()
        .await
        .unwrap();

    let hit_count = get_hits_resp
        .text()
        .await
        .unwrap();

    // demo data starts with 6 hits, the unit tests add 1 more,
    // and the above code adds one more: 6+1+1 == 8
    // 
    // A previous run without TLS will add 1 more, so it may also be 9
    assert!(hit_count == "8" || hit_count == "9");
}