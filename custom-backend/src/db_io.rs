use std::env; 
use axum::body::Body;
use axum::response::Response;
use axum::{
    response::Html, 
    Json
};
use chrono::prelude::*;
use mysql::*;
use mysql::prelude::*;
use tracing::info;
use crate::err_handling::WebsiteError;

#[derive(Debug, serde::Deserialize)] 
pub struct GuestbookEntry {
    name: String,
    note: String,
}
#[derive(Debug, serde::Serialize)]
pub struct GuestbookEntryStamped {
    // keeping time_stamp as NaiveDateTime for DB I/O,
    // and for time-value sorting to by done by the DB on query
    time_stamp: NaiveDateTime,  
    name: String,
    note: String,
}

// This struct exists for organinzing all the JSON 
// guestbook entries for transmission to the client into a
// larger JSON object
#[derive(Debug, serde::Serialize)]
pub struct Guestbook {
    guestbook: Vec<GuestbookEntryStamped>,
}

#[derive(Debug, serde::Deserialize)]
pub struct WebpageHit {
    user_agent: String,
    time_stamp: NaiveDateTime,
}

fn get_db_conn() -> Result<mysql::Pool, UrlError> {
    
    // gets URL from the environment to preserve security,
    // since it contains a plain-text password
    let url = env::var_os("DB_URL")
        .unwrap()
        .into_string()
        .unwrap();

    let opts = Opts::from_url(&url)?;

    Ok(Pool::new(opts).unwrap()) // unwrap() b/c infallible
}


// it is a simpler, albeit slower, design to establish the connection every
// time the function is called
pub async fn get_guestbook() -> Result<Json::<Guestbook>, WebsiteError> {

    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;
    let guestbook_table = conn.query_map(
        "
        SELECT dateSubmitted, guestName, guestNote 
        FROM guestbook
        ORDER BY dateSubmitted DESC", // let the DB do the sorting
        |(time_stamp, name, note)| {
            GuestbookEntryStamped {time_stamp, name, note}
        }
    )?;

    Ok(Json(Guestbook {guestbook: guestbook_table}))
}

pub async fn update_guestbook(Json(form_entry): Json<GuestbookEntry>) -> Result<Html<String>, WebsiteError> {

    // db connection setup
    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;

    // converting in this scope so that the name is in scope
    // for printing name on entry to server console (for fun)
    let entry_name = if form_entry.name.is_empty() {
        "(anonymous)".to_string()
    } else {
        form_entry.name
    };

    // return value needs to be caught so that type can be annotated
    let _: Option<Row> = conn.exec_first(
        r"INSERT INTO guestbook (dateSubmitted, guestName, guestNote)
                VALUES (UTC_TIMESTAMP(), :name, :note)",
        params! {
            "name" => &entry_name, 
            "note" => form_entry.note
        }
    )?;

    info!("New entry in the guestbook from {}!", entry_name);
    Ok(Html(String::from(
        "<!DOCTYPE html>\n\
        <html><head>\n\
        <title>Entry Received!</title>\n\
        </head>\n\
        <p>Thanks for leaving a note on my website!</p>\n\
        </html>"
    )))
}


// adds new hit info to database
pub async fn get_hit_count() -> Result<String, WebsiteError> {
    
    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;
        
    match conn.query_first::<String, &str>(
        r"SELECT COUNT(*) AS hit_count FROM hitLog"
    )? {
        Some(hits_count) => Ok(hits_count),
        None => Ok(String::from("0"))
    }     
}


pub async fn log_hit(Json(page_hit): Json<WebpageHit>) -> Result<Response, WebsiteError> {
    
    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;
        
    // using transaction with defined isolation level to prevent race conditions
    let tx_options = TxOpts::default()
        .set_isolation_level(Some(
            IsolationLevel::Serializable
        ));

    let mut tx = conn
        .start_transaction(tx_options)?;   
        
    // INSERT statements return a string with the number of rows affected, 
    // warnings, and duplicates.
    // In this context this doesn't matter, except for type annotations.
    tx.exec_first::<String, &str, Params>(
        r"INSERT INTO hitLog (hitTime, userAgent) VALUES (:time_stamp, :user_agent);",
        params! {
            "time_stamp" => page_hit.time_stamp, 
            "user_agent" => page_hit.user_agent
        }
    )?;
    tx.commit()?;
    
    Ok(Response::new(Body::empty()))  // return 200 OK
}