use tokio;
use reqwest::{self, StatusCode};
use mysql_common::serde_json;
use custom_backend::{
    utils::init_utils::get_env_var,
    srv_io::db_io::WebpageHit
};


#[tokio::main]
async fn main() {

    let svr_sock = get_env_var("SERVER_SOCKET").unwrap();
    let req_client = reqwest::Client::new();
    let new_hit = WebpageHit::default();

    let post_hit_resp = req_client
        .post(format!("http://{svr_sock}/hits"))
        .header(reqwest::header::CONTENT_TYPE,"application/json")
        .body(serde_json::to_string(&new_hit).unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(post_hit_resp.status(), StatusCode::OK);

    let get_hits_resp = req_client
        .get(format!("http://{svr_sock}/hits"))
        .send()
        .await
        .unwrap();

    let hit_count = get_hits_resp
        .text()
        .await
        .unwrap();

    // demo data starts with 6 hits, the unit tests add 2 more,
    // and the above code adds one more: 6+2+1 == 9
    assert_eq!(hit_count, "9");
}