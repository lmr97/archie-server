use std::env; 
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
    note: String
}
#[derive(Debug, serde::Serialize)]
pub struct GuestbookEntryStamped {
    // keeping time_stamp as NaiveDateTime for DB I/O,
    // and for time-value sorting to by done by the DB on query
    time_stamp: NaiveDateTime,  
    name: String,
    note: String
}

// This struct exists for organinzing all the JSON 
// guestbook entries for transmission to the client into a
// larger JSON object
#[derive(Debug, serde::Serialize)]
pub struct Guestbook {
    guestbook: Vec<GuestbookEntryStamped>
}


fn get_db_conn() -> Result<mysql::Pool, UrlError> {
    
    // gets URL from the environment to preserve security,
    // since it contains a plain-text password
    let url = env::var_os("DB_URL")
        .expect("Database URL variable not found in environment.")
        .into_string()
        .unwrap();  // this failure should crash the server, it cannot run without it

    let opts_res = Opts::from_url(&url);

    match opts_res {
        Ok(opts) => {
            Ok(Pool::new(opts).unwrap()) // unwrap() b/c infallible
        },  
        Err(err) => { Err(err) }
    }
}


// it is a simpler, albeit slower, design to establish the connection every
// time the function is called
#[tracing::instrument(ret)]
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

#[tracing::instrument(ret)]
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
    let _: Vec<Row> = conn.exec(
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


// Updates the list of timestamps (corresponding to home page hits),
// then returns the total number of hits as a JSON response.
// This is because, on this website, hits are defined by how many
// GET requests there are to /hits.
#[tracing::instrument(ret)]
pub async fn update_hits() -> Result<String, WebsiteError> {
    
    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;
        
    let tx_options = TxOpts::default()
        .set_isolation_level(Some(
            IsolationLevel::Serializable
        ));

    // unwrapping b/c the earlier ? would have caught any 
    // conn issues, and I know that the transaction options are
    // allowed for the database
    let mut tx = conn
        .start_transaction(tx_options)?;   
        
        // INSERT statements return a string with the number of rows effected, 
        // warnings, and duplicates.
        // In this context it doesn't matter, except for type annotations
        tx.exec_first::<String, &str, Params>(
            r"INSERT INTO hitsLog (hitTime) VALUES (UTC_TIMESTAMP());",
            Params::Empty
        )?;
        tx.commit()?; // as before, conn issues would have been handled previously

        // get the number as a string, because that has IntoResponse implemented
        if let Some(hits_count) = conn.query_first::<String, &str>(
            r"SELECT COUNT(*) AS hits_count FROM hitsLog"
        )? {
            Ok(hits_count)
        }
        else {
            Ok(String::from("0"))
        }
                
}