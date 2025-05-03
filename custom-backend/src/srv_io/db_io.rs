use std::env; 
use axum::{
    body::Body,
    response::{Html, IntoResponse, Response}, 
    Json
};
use chrono::prelude::*;
use mysql::*;
use mysql::prelude::*;
use tracing::{info, debug, error};
use crate::utils::err_handling::make_500_resp;


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
    time_stamp: NaiveDateTime,
    user_agent: String,
}

// wrapper to implement IntoResponse
#[derive(Debug)]
pub enum DbError { 
    UrlError(mysql::UrlError),
    GenError(mysql::Error), 
    
}

impl From<mysql::Error> for DbError {
    fn from(mysql_err: mysql::Error) -> Self {
        Self::GenError(mysql_err)
    }
}

impl From<mysql::UrlError> for DbError {
    fn from(url_err: mysql::UrlError) -> Self {
        Self::UrlError(url_err)
    }
}

impl IntoResponse for DbError {

    fn into_response(self) -> Response {

        error!("Error in database I/O: {self:?}");
        make_500_resp()
    }
}


fn get_db_conn() -> Result<mysql::Pool, mysql::Error> {
    
    // gets URL from the environment to preserve security,
    // since it contains a plain-text password
    let url = env::var_os("DB_URL")
        .unwrap()       // These two unwraps do not panic
        .into_string()
        .unwrap();
    
    let opts = Opts::from_url(&url)?;

    Pool::new(opts)
}


// it is a simpler, albeit slower, design to establish the connection every
// time the function is called
pub async fn get_guestbook() -> Result<Json::<Guestbook>, DbError> {

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

    debug!("GET /guestbook successful.");

    Ok(Json(Guestbook {guestbook: guestbook_table}))
}

pub async fn update_guestbook(Json(form_entry): Json<GuestbookEntry>) -> Result<Html<String>, DbError> {

    // db connection setup
    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;

    // return value needs to be caught so that type can be annotated
    let _: Option<Row> = conn.exec_first(
        r"INSERT INTO guestbook (dateSubmitted, guestName, guestNote)
                VALUES (UTC_TIMESTAMP(), :name, :note)",
        params! {
            "name" => &form_entry.name, 
            "note" =>  form_entry.note
        }
    )?;

    info!("New entry in the guestbook from {}!", form_entry.name);
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
pub async fn get_hit_count() -> Result<String, DbError> {
    
    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;
        
    let hits = match conn.query_first::<String, &str>(
        r"SELECT COUNT(*) AS hit_count FROM hitLog"
    )? {
        Some(hits_count) => hits_count,
        None => String::from("0")
    };

    debug!("Page hit count retrieved.");

    Ok(hits)   
}


pub async fn log_hit(Json(page_hit): Json<WebpageHit>) -> Result<Response, DbError> {

    let buf_pool = get_db_conn()?;
    let mut conn = buf_pool.get_conn()?;

    // this function runs async of get_hit_count(), which is called immediately after this one
    // through a GET request to /hits. In practice, this means the hit count it returned was
    // 1 behind the DB.
    // So, I'm setting a write lock to block the GET that comes on the heels of this INSERT.
    // There is some slight overhead for this, unsuprisingly, but that's acceptable.
    conn.query_first::<String, &str>("LOCK TABLE hitLog WRITE")?;
    debug!("table lock successful");
    conn.exec_first::<String, &str, Params>(
        r"INSERT INTO hitLog (hitTime, userAgent) VALUES (:time_stamp, :user_agent);",
        params! {
            "time_stamp" => page_hit.time_stamp, 
            "user_agent" => &page_hit.user_agent
        }
    )?;
    debug!("prep'd statement successful");
    conn.query_first::<String, &str>("UNLOCK TABLES")?;
    debug!("table unlock successful");
    
    info!("New visit from: {}", page_hit.user_agent);

    Ok(Response::new(Body::empty()))  // return 200 OK
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn log_hit_normal() {
        
        let page_hit_normal = WebpageHit {
            time_stamp: Utc::now().naive_utc(),
            user_agent: String::from("Mozilla Firefox user")
        };

        log_hit(Json(page_hit_normal)).await.unwrap();
    }

    #[tokio::test]
    async fn log_hit_no_ua() {

        let page_hit_no_ua = WebpageHit {
            time_stamp: Utc::now().naive_utc(),
            user_agent: String::from("")
        };

        log_hit(Json(page_hit_no_ua)).await.unwrap();
    }
}