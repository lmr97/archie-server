// This test tests input to the Letterboxd app are sent without error, 
// and that output from the Letterboxd app is handled appropriately.
// These tests are simply adapted versions of the library tests for lb_app_io.rs.
//
// This test is intended to be run from the custom-backend directory, and will crash

use std::{fs::OpenOptions, io::Read};
use tokio;
use futures::StreamExt;
use mysql_common::serde_json;
use eventsource_stream::Eventsource;
use custom_backend::types::lb_app_types::ListRow;
mod client_config;

#[tokio::main]
async fn main(){

    let (protocol, base_url) = client_config::get_base_url();
    let client = client_config::config_client(protocol);

    too_long_list_req(&client, base_url.clone()).await;
    lb_server_down_req(&client, base_url.clone()).await;
    bad_list_req(&client, base_url.clone()).await;
    bad_attr_req(&client, base_url.clone()).await;
    req_to_crashed_app(&client, base_url.clone()).await;
    no_attr_req(&client, base_url.clone()).await;
    big_list_req(&client, base_url.clone()).await;
}


// sends request and then gets stream as a Vec of Events
async fn extract_events(client: &reqwest::Client, url: String) -> Vec<eventsource_stream::Event> {

    let mut event_stream= client
        .get(url)
        .send()
        .await
        .unwrap()
        .bytes_stream()
        .eventsource();

    let mut event_data: Vec<eventsource_stream::Event> = vec![];
    while let Some(event_res) = event_stream.next().await {
        match event_res {
            Ok(event) => {
                // break the loop at the end of SSE stream
                if event.data == "done!" {
                    break;
                }
                event_data.push(event);
            }
            Err(_) => {
                panic!("Error in event stream");
            }
        }
    }

    event_data
}


async fn too_long_list_req(client: &reqwest::Client, mut url: String) {

    // This is the data used in the URL query, in JSON format:
    // {
    //     list_name: "list-too-long",
    //     author_user: "user_exists",
    //     attrs: ["casting", "watches", "likes"]    // all valid attributes
    // }

    url.push_str(
        "/lb-list-conv/conv?\
        list_name=list-too-long&\
        author_user=user_exists&\
        attrs=casting&\
        attrs=watches&\
        attrs=likes"
    );

    let event_data = extract_events(client, url).await;

    assert!(event_data[0].event == "error");
    assert!(event_data[0].data.contains("403 FORBIDDEN"));
}


async fn lb_server_down_req(client: &reqwest::Client, mut url: String){

    // for when Letterboxd.com is down, testing sending 502 status to client
    url.push_str(
        "/lb-list-conv/conv?\
        list_name=server-down&\
        author_user=some_user&\
        attrs=stuff"
    );

    let event_data = extract_events(client, url).await;

    assert!(event_data[0].event == "error");
    assert!(event_data[0].data.contains("502 BAD GATEWAY"));
}


async fn bad_list_req(client: &reqwest::Client, mut url: String){

    // if the user doesn't exist, the list doesn't, 
    // because it'll turn up a bad URL regardless.
    // So it only matters whether the list exists or not.

    url.push_str(
        "/lb-list-conv/conv?\
        list_name=list-no-exist&\
        author_user=user_may_exist&\
        attrs=casting&\
        attrs=watches&\
        attrs=likes"
    );

    let event_data = extract_events(client, url).await;

    assert!(event_data[0].event == "error");
    assert!(event_data[0].data.contains("422 UNPROCESSABLE CONTENT"));
}


async fn req_to_crashed_app(client: &reqwest::Client, mut url: String) {

    // if the Letterboxd app sends a crash error (500), make sure the server can detect
    // the 500 error sent over, and wraps it up in an event of type "error"
    url.push_str(
        "/lb-list-conv/conv?\
        list_name=this-hurts-you&\
        author_user=user_may_exist&\
        attrs=casting&\
        attrs=watches&\
        attrs=likes"
    );

    let event_data = extract_events(client, url).await;

    assert!(event_data[0].event == "error");
    assert!(event_data[0].data.contains("500 INTERNAL SERVER ERROR"));
}


async fn bad_attr_req(client: &reqwest::Client, mut url: String) {

    url.push_str(
        "/lb-list-conv/conv?\
        list_name=list-exists&\
        author_user=user_exists&\
        attrs=casting&\
        attrs=bingus&\
        attrs=likes"
    );

    let event_data = extract_events(client, url).await;

    assert!(event_data[0].event == "error");
    assert!(event_data[0].data.contains("422 UNPROCESSABLE CONTENT"));

}


async fn no_attr_req(client: &reqwest::Client, mut url: String) {

    url.push_str(
        "/lb-list-conv/conv?\
        list_name=list-exists&\
        author_user=user_exists&\
        attrs=none"
    );

    let event_data = extract_events(client, url).await;

    let correct_rows = vec![
        ListRow {
            total_rows: 5,
            row_data: String::from("Title,Year"),
        },
        ListRow {
            total_rows: 5,
            row_data: String::from("2001: A Space Odyssey,1968"),
        },
        ListRow {
            total_rows: 5,
            row_data: String::from("Blade Runner,1982"),
        },
        ListRow {
            total_rows: 5,
            row_data: String::from("The Players vs. Ángeles Caídos,1969"),
        },
        ListRow {
            total_rows: 5,
            row_data: String::from("8½,1963"),
        },
    ];

    let received_rows: Vec<ListRow> = event_data
        .iter()
        .map(| ev | { 
            // this makes sure that the ListRow serialization is camelCase
            assert!(ev.data.contains("totalRows"));
            assert!(ev.data.contains("rowData"));
            serde_json::from_str::<ListRow>(&ev.data)
                .unwrap() 
        })
        .collect();

    assert_eq!(correct_rows, received_rows);
}


async fn big_list_req(client: &reqwest::Client, mut url: String){

    // This exists to test very long rows, with a very long list (~1.5k films)

    url.push_str(
        "/lb-list-conv/conv?\
        list_name=the-big-one&\
        author_user=user_exists&\
        attrs=all-of-em"
    );

    let mut test_file_reader = OpenOptions::new()
        .read(true)
        .open("../test-helpers/big-list-test.csv")
        .expect(
            "Could not find `big-list-test.csv`; are you running this \
            from the `custom-backend` directory?"
        );

    let mut correct_list = String::new();
    test_file_reader
        .read_to_string(&mut correct_list)
        .unwrap();

    let event_data = extract_events(client, url).await;

    // Extract row data itself, not entire ListRow structs
    let received_list  = event_data
        .iter()
        .map(| ev | { 
            // this makes sure that the ListRow serialization is camelCase
            assert!(ev.data.contains("totalRows"));
            assert!(ev.data.contains("rowData"));
            let li = serde_json::from_str::<ListRow>(&ev.data)
                .unwrap();
            li.row_data
        })
        .collect::<Vec<String>>()
        .join("");

    assert_eq!(correct_list, received_list);
}