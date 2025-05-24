use tokio;
use reqwest::{self, StatusCode};
use mysql_common::serde_json;
use custom_backend::{
    utils::init_utils::{
        get_env_var, 
        process_cli_args, 
        RunMode
    },
    srv_io::db_io::WebpageHit
};


#[tokio::main]
async fn main() {

    /* Sort out protocol to use */
    let protocol = match process_cli_args().unwrap() {
        RunMode::NoTls => "http",
        _ => "https"
    };
    let domain = get_env_var("CLIENT_SOCKET").unwrap();
    let url    = format!("{protocol}://{domain}/hits");

    let req_client = reqwest::Client::new();
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
    assert_eq!(hit_count, "8");
}