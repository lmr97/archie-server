use std::env; 
use axum::{
    body::Body, http::StatusCode, response::{Html, IntoResponse, Response}, Json
};
use mysql::*;
use mysql::prelude::*;
use mysql_common::chrono::Utc;
use tracing::{info, debug, error};
use crate::utils::err_handling::make_500_resp;
use crate::types::db_io_types::*;

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

#[derive(Debug, PartialEq)]
pub enum UserError { 
    NameTooLong,
    NoteTooLong,
}

// a composite error type to allow both user and DB errors
// to be raised from a function

#[derive(Debug)]
pub enum DbOrUserError {
    DbError(DbError),
    UserError(UserError),
}

impl From<mysql::Error> for DbOrUserError {
    fn from(mysql_err: mysql::Error) -> DbOrUserError {
        DbOrUserError::DbError(DbError::from(mysql_err))
    }
}

impl From<UserError> for DbOrUserError {
    fn from(u_err: UserError) -> DbOrUserError {
        DbOrUserError::UserError(u_err)
    }
}

impl IntoResponse for DbOrUserError {
    fn into_response(self) -> Response {
        
        match self {
            DbOrUserError::DbError(e) => DbError::into_response(e),
            DbOrUserError::UserError(ue) => {

                let (entry_field, db_limit) = match ue {
                    UserError::NameTooLong => { ("Name", 100)  },
                    UserError::NoteTooLong => { ("Note", 1000) }
                };

                let too_long_msg = Html(format!(
                    "<!DOCTYPE html>\n\
                    <html>\n\
                    <head>\n\
                    \t<title>413 Payload Too Large</title>\n\
                    \t<link rel=\"stylesheet\" href=\"../static/styles/err-style.css\">\n\
                    \t<meta charset=\"utf-8\">\n\
                    </head>\n\
                    <h1>{} too long!</h1>\n\
                    <p>The database limits this field to {} bytes.</p>\n\
                    <p>Try again with a shorter entry for that field!</p>\n\
                    </html>\n",
                    entry_field,
                    db_limit
                ));
            
                let mut err_resp = Html(too_long_msg).into_response();
                *err_resp.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;

                err_resp
            }
        }
    }
}




fn get_db_conn_pool() -> Result<mysql::Pool, mysql::Error> {
    
    // gets URL from the environment to preserve security,
    // since it contains a plain-text password
    let url = env::var_os("DB_URL")
        .unwrap()       // These two unwraps do not panic
        .into_string()
        .unwrap();

    debug!("DB URL to connect with: {url}");

    let opts = Opts::from_url(&url)?;

    Pool::new(opts)
}


// it is a simpler, albeit slower, design to establish the connection every
// time the function is called
pub async fn get_guestbook() -> Result<Json::<Guestbook>, DbError> {

    let buf_pool = get_db_conn_pool()?;
    let mut conn = buf_pool.get_conn()?;
    
    let guestbook_table = conn.query_map(
        "
        SELECT id, dateSubmitted, guestName, guestNote 
        FROM guestbook
        ORDER BY dateSubmitted DESC", // let the DB do the sorting
        |(id, time_stamp, name, note)| {
            GuestbookEntry {id: Some(id), time_stamp: Some(time_stamp), name, note}
        }
    )?;

    debug!("GET /guestbook successful.");

    Ok(Json(Guestbook {guestbook: guestbook_table}))
}

pub async fn update_guestbook(Json(mut form_entry): Json<GuestbookEntry>) -> Result<Json<EntryReceipt>, DbOrUserError> {

    // the first two conditionals are redundancies to catch entries that exceed
    // hard-coded VARCHAR limits, since the client-side Javascript is designed
    // to catch them as well. But they are included here just in case the API
    // is accessed outside the browser's JS.
    
    if form_entry.name.len() > 150 {  // MySQL uses bytes for length definitions, as does String.len() in Rust
        
        return Err(DbOrUserError::UserError(UserError::NameTooLong));
    }
    
    if form_entry.note.len() > 1000 {  // MySQL uses bytes for length definitions, as does String.len() in Rust
        
        return Err(DbOrUserError::UserError(UserError::NoteTooLong));
    }


    // db connection setup
    let buf_pool = get_db_conn_pool()?;
    let mut conn = buf_pool.get_conn()?;

    // also redundant, since this is delegated to the client JS, 
    // but included just in case
    if form_entry.name.is_empty() { form_entry.name = String::from("(anonymous)"); }

    // while the UTC_TIMESTAMP() in the query will theorietically not be precisely
    // the same as Utc::now() invoked here, they will be close enough
    // and it simplifies the code.
    form_entry.time_stamp = Some(Utc::now().naive_utc());
    let _: Option<Row> = conn.exec_first(
        r"INSERT INTO guestbook (dateSubmitted, guestName, guestNote)
                VALUES (UTC_TIMESTAMP(), :name, :note)",
        params! {
            "name"      => &form_entry.name, 
            "note"      => &form_entry.note
        }
    )?;

    let new_entry_id: String = match conn.query_first(
        "SELECT LAST_INSERT_ID()")? {
        Some(id) => id,
        None => { String::from("0") }
    };

    info!("New entry in the guestbook from {}!", form_entry.name);
    Ok(Json(EntryReceipt {
        // nwrapping here is safe, because I set this field to Some()
        // a few lines before in this function
        time_stamp: form_entry.time_stamp.unwrap(), 
        id: new_entry_id,
    }))
}


// adds new hit info to database
pub async fn get_hit_count() -> Result<String, DbError> {
    
    let buf_pool = get_db_conn_pool()?;
    let mut conn = buf_pool.get_conn()?;
    
    conn.query_first::<String, &str>("LOCK TABLE hitLog READ")?;

    let hits = match conn.query_first::<String, &str>(
        r"SELECT COUNT(*) AS hit_count FROM hitLog"
    )? {
        Some(hits_count) => hits_count,
        None => String::from("0")
    };

    conn.query_first::<String, &str>("UNLOCK TABLES")?;
    debug!("Page hit count retrieved.");

    Ok(hits)   
}


pub async fn log_hit(Json(page_hit): Json<WebpageHit>) -> Result<Response, DbError> {

    let buf_pool = get_db_conn_pool()?;
    let mut conn = buf_pool.get_conn()?;

    // this function runs async of get_hit_count(), which is called immediately after this one
    // through a GET request to /hits. In practice, this means the hit count it returned was
    // 1 behind the DB.
    // So, I'm setting a read/write lock on the table to block the GET that comes on the heels 
    // of this INSERT. There is some slight overhead for this, unsuprisingly, but that's 
    // acceptable to me.
    conn.query_first::<String, &str>("LOCK TABLE hitLog READ")?;
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
    use axum::http::StatusCode;
    use mysql_common::chrono::{Utc, NaiveDateTime, SubsecRound};

    #[tokio::test]
    async fn hit_counting() {

        let hits = get_hit_count().await.unwrap();

        // there are precisely 6 hits in the demo DB's log,
        // and since a lock is requested on the table, 
        assert_eq!(hits, "6");
    }

    #[tokio::test]
    async fn log_hit_normal() -> Result<(), DbError>{
        
        let page_hit_normal = WebpageHit {
            time_stamp: Utc::now()
                .naive_utc()
                .trunc_subsecs(0),    // truncation happens after db insertion/retrieval
            user_agent: String::from("a user agent string no one uses")
        };

        let sent_resp = log_hit(Json(page_hit_normal.clone())).await.unwrap();
        assert_eq!(sent_resp.status(), StatusCode::OK);  // make sure a successful call sends 200

        // connect
        let mut conn = get_db_conn_pool()?.get_conn()?;

        // no other entries in the demo database have a user agent like the above
        let latest_hit = conn.query_map(
            format!(
                "SELECT hitTime, userAgent FROM hitLog 
                WHERE hitTime = STR_TO_DATE('{}', '%Y-%m-%d %H:%i:%S')
                AND userAgent = '{}'", 
                /* note that the MYSQL date format does not follow strftime format */
                page_hit_normal.time_stamp.format("%Y-%m-%d %H:%M:%S").to_string(),
                page_hit_normal.user_agent,
            ),
            |(hit_time, user_agent): (NaiveDateTime, String)| {
                WebpageHit { time_stamp: hit_time, user_agent, }
            }
        )?;

        assert_eq!(latest_hit[0], page_hit_normal);

        Ok(())
    }

    #[tokio::test]
    async fn getting_guestbook() -> Result<(), DbError> {

        let demo_guestbook = Guestbook {
            guestbook: vec![
                GuestbookEntry {
                    id: Some(String::from("4")),
                    time_stamp: Some(NaiveDateTime::parse_from_str(
                        "2025-04-20 13:03:59", "%Y-%m-%d %H:%M:%S").unwrap()),
                    name:       String::from("约翰·塞纳"),
                    note:       String::from("我很喜欢冰淇淋")
                },
                GuestbookEntry {
                    id: Some(String::from("3")),
                    time_stamp: Some(NaiveDateTime::parse_from_str(
                        "2025-03-13 03:37:05", "%Y-%m-%d %H:%M:%S").unwrap()),
                    name:       String::from("Linus"),
                    note:       String::from("nice os choice!")
                },
                GuestbookEntry {
                    id: Some(String::from("2")),
                    time_stamp: Some(NaiveDateTime::parse_from_str(
                        "2025-02-28 04:30:57", "%Y-%m-%d %H:%M:%S").unwrap()),
                    name:       String::from("(anonymous)"),
                    note:       String::from("you'll never know...")
                },
                
                GuestbookEntry {
                    id: Some(String::from("1")),
                    time_stamp: Some(NaiveDateTime::parse_from_str(
                        "2025-02-28 04:22:49", "%Y-%m-%d %H:%M:%S").unwrap()),
                    name:       String::from("Ada"),
                    note:       String::from("It's so nice to be here!")
                },
            ]
        };

        let gotten_guestbook = get_guestbook().await.unwrap();
        assert_eq!(gotten_guestbook.0, demo_guestbook);

        Ok(())
    }

    #[tokio::test]
    async fn post_null_entry() -> Result<(), DbOrUserError> {

        // allows for SQL query to filter all entries later
        // than the start of this function
        let proc_start = Utc::now().naive_utc();
        let mut null_entry = GuestbookEntry {
            id: Some(String::from("7")), 
            time_stamp: None,
            name: String::new(),
            note: String::new()
        };

        // Running this in series, this entry should be ID == 7,
        // since the last auto-increment ID insert in this connection
        // since DB initialization (and that's the counter measured)
        // happened with posting the 6th hit, so the auto-increment counter
        // is now at 7.
        let receipt = update_guestbook(Json(null_entry.clone())).await.unwrap();
        assert_eq!(receipt.0.id, "7");

        // check to see if the entry was posted with 
        let mut conn = get_db_conn_pool()?.get_conn()?;

        // no other entries in the demo database have a null user agent
        let fetched_entry = conn.query_map(
            format!(
                "SELECT id, guestName, guestNote FROM guestbook
                WHERE dateSubmitted >= STR_TO_DATE('{}', '%Y-%m-%d %H:%i:%S')
                AND guestName = '(anonymous)'", 
                proc_start.format("%Y-%m-%d %H:%M:%S").to_string()
            ),
            |(id, name, note): (String, String, String)| {
                // timestamp not needed if the query finds an entry in terms of proc_start
                // ID should be 5.
                GuestbookEntry { id: Some(id), time_stamp: None, name, note }
            }
        )?;

        null_entry.name = String::from("(anonymous)");
        assert_eq!(fetched_entry[0], null_entry);

        Ok(())
    }

    #[tokio::test]
    async fn post_valid_entry() -> Result<(), DbOrUserError> {

        // allows for SQL query to filter all entries later
        // than the start of this function
        let proc_start = Utc::now().naive_utc();

        // gonna get real weird with it
        let valid_entry = GuestbookEntry {
            id: Some(String::from("8")), 
            time_stamp: None,
            name: String::from("Lettuce % % \\% \\' break some sTuff ⌠ 	⌡ 	⌢ 	⌣ 	⌤"),
            note: String::from(
                "ᏣᎳᎩ ᎦᏬᏂᎯᏍᏗ (Cherokee!) \n\\\\% %%' ''\\n\
                മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി \
                ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് \
                വിവേകബുദ്ധിയും മനസാക്ഷിയും സിദ്ധമായിരിക്കുന്നത്‌ \
                (this says 'All human beings are born free and equal in dignity and rights. \
                They are endowed with reason and conscience and should act towards one \
                another in a spirit of brotherhood.' in Malayalam. It comes from the \
                UN's Universal Declaration on Human Rights)"
            ),
        };

        // run in series, this entry should have ID == 8, see post_null_entry()
        let receipt = update_guestbook(Json(valid_entry.clone())).await.unwrap();
        assert_eq!(receipt.0.id, "8");  

        // check to see if the entry was posted with 
        let mut conn = get_db_conn_pool()?.get_conn()?;

        // no other entries in the demo database have a null user agent
        let fetched_entry = conn.query_map(
            format!(
                "SELECT id, guestName, guestNote FROM guestbook
                WHERE dateSubmitted >= STR_TO_DATE('{}', '%Y-%m-%d %H:%i:%S')
                AND guestName LIKE 'Lettuce%'", 
                proc_start.format("%Y-%m-%d %H:%M:%S").to_string()
            ),
            |(id, name, note): (String, String, String)| {
                // timestamp not needed if the query finds an entry in terms of proc_start
                GuestbookEntry { id: Some(id), time_stamp: None, name, note }
            }
        )?;

        assert_eq!(fetched_entry[0], valid_entry);

        Ok(())
    }

    #[tokio::test]
    async fn post_overlong_entry_note()-> Result<(), DbOrUserError> {

        // gonna get real weird with it
        let overlong_entry = GuestbookEntry {
            id: None, 
            time_stamp: None,
            name: String::from("A resonable name"),
            note: String::from(
                "ᏣᎳᎩ ᎦᏬᏂᎯᏍᏗ (this is Cherokee!) \n\\\\% %%' ''\n\
                മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും അന്തസ്സോടും സ്വാതന്ത്ര്യത്തോടുംകൂടി \
                ജനിച്ചിട്ടുള്ളവരാണ്‌. അന്യോന്യം ഭ്രാതൃഭാവത്തോടെ പെരുമാറുവാനാണ്‌ മനുഷ്യന് \
                വിവേകബുദ്ധിയും മനസാക്ഷിയും സിദ്ധമായിരിക്കുന്നത്‌\
                (this says 'All human beings are born free and equal in dignity and rights. \
                They are endowed with reason and conscience and should act towards one \
                another in a spirit of brotherhood.' in Malayalam. It comes from the \
                UN's Universal Declaration on Human Rights)\n\
                Let's stick with this and go further. We need to make sure we have this \
                data exceed 1KB. And now it does, with all these extra characters \
                to put it over the finsh line."
            ),
        };

        // this part is good as long as it doesn't panic (which it would on anOk variant here)
        let name_len_err = update_guestbook(Json(overlong_entry.clone()))
            .await
            .unwrap_err();

        // gotta do it the hard way, because mysql::Error, a part of DbOrUserError,
        // doesn't impl PartialEq
        match name_len_err {
            DbOrUserError::DbError(e) => panic!("Wrong error! {:?}", e),
            DbOrUserError::UserError(e) => {
                assert_eq!(e, UserError::NoteTooLong);
                Ok(())
            }
        }
    }
    
    #[tokio::test]
    async fn post_overlong_entry_name()-> Result<(), DbOrUserError> {

        // gonna get real weird with it
        let overlong_name = GuestbookEntry {
            id: None, 
            time_stamp: None,
            name: String::from(
                "A name മനുഷ്യരെല്ലാവരും തുല്യാവകാശങ്ങളോടും that is too ᎦᏬᏂᎯᏍᏗ long. \
                so long, in fact, I needed to add all this stuff!"),
            note: String::from("a brief note"),
        };

        // this part is good as long as it doesn't panic (which it would on anOk variant here)
        let name_len_err = update_guestbook(Json(overlong_name.clone()))
            .await
            .unwrap_err();

        // gotta do it the hard way, because mysql::Error, a part of DbOrUserError,
        // doesn't impl PartialEq
        match name_len_err {
            DbOrUserError::DbError(e) => panic!("Wrong error! {:?}", e),
            DbOrUserError::UserError(e) => {
                assert_eq!(e, UserError::NameTooLong);
                Ok(())
            }
        }
    }
    
}