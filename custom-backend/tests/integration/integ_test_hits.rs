use tokio;
use reqwest::{self, Client, StatusCode};
use mysql_common::serde_json;
use custom_backend::types::db_io_types::WebpageHit;
use mysql_common::chrono::{NaiveDateTime, Utc, SubsecRound};
mod client_config;

// use the JS styling to make sure the deserialization converts correctly
// when taken from the client
#[derive(serde::Serialize)]
#[allow(non_snake_case)]
struct WebpageHitJs {
    timeStamp: NaiveDateTime,
    userAgent: String,
}

impl Default for WebpageHitJs {
        fn default() -> WebpageHitJs {
            WebpageHitJs { 
                timeStamp: Utc::now()
                    .naive_utc()
                    .trunc_subsecs(0), 
                userAgent: String::from("Mozilla user agent") 
            }
        }
    }


#[tokio::main]
async fn main() {

    let (protocol, mut url) = client_config::get_base_url();
    url.push_str("/hits");
    let req_client = client_config::config_client(protocol);

    let new_hit      = WebpageHit::default();
    let new_hit_js = WebpageHitJs::default();
    let hit_serial   = serde_json::to_string(&new_hit).unwrap();
    let hit_serial_js = serde_json::to_string(&new_hit_js).unwrap();

    let hit_count    = test_posting_hit(req_client.clone(), url.clone(), hit_serial).await;
    let hit_count_js = test_posting_hit(req_client, url, hit_serial_js).await;

    // demo data starts with 6 hits, the unit tests add 1 more,
    // and the above code adds one more: 6+1+1 == 8
    // 
    // A previous run without TLS will add 1 more, so it may also be 9
    assert!(hit_count == "8" || hit_count == "9");
    assert!(hit_count_js == "9" || hit_count_js == "10");
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