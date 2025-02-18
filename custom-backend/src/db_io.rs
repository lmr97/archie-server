use std::env; 
use axum::{
    response::IntoResponse,
    Json
};
use chrono::prelude::*;
use mysql::*;
use mysql::prelude::*;


#[derive(Debug, serde::Deserialize, serde::Serialize)] 
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


fn get_db_conn() -> Option<mysql::Pool> {
    
    // gets URL from the environment to preserve security,
    // since it contains a plain-text password
    let url = env::var_os("DB_URL")
        .expect("Database URL variable not found in environment.")
        .into_string()
        .unwrap();  // this failure should crash the server

    let opts_res = Opts::from_url(&url);

    match opts_res {
        Ok(opts) => {
            Some(Pool::new(opts).unwrap()) // unwrap() b/c infallible
        },  
        Err(_) => { None }
    }
}


fn send_500_html() -> axum::response::Html<String> {
    let get_html_res = std::fs::read_to_string(
        "/home/martin/archie-server/static/errors/500.html"
    );

    let html_string= match get_html_res {
        Ok(html_string) => { html_string },
        Err(_) => {
            println!("ERROR: 500.html not found at usual location, sending unpretty version.");
            format!("\
                <!DOCTYPE html>\n\
                <html><head>\n\
                <title>500 Error</title>\n\
                </head>\n\
                <p>500 Error: Internal Server Error</p>\n\
                <p>This is most likely due to issues with the database.</p>\n\
                </html>"
            )
        }
    };

    axum::response::Html(html_string.clone())
}



// it is a simpler, albeit slower, design to make the connection every
// time the function is called
pub async fn get_guestbook() -> Json {

    let buf_pool = match get_db_conn() {
        Some(bp) => { bp },
        None => { return send_500_json(); }
    };

    let conn_res = buf_pool.get_conn();
    match conn_res {
        Ok(mut conn) => {
            let query_res = conn.query_map(
                "
                SELECT dateSubmitted, guestName, guestNote 
                FROM guestbook
                ORDER BY dateSubmitted DESC", // let the DB do the sorting
                |(time_stamp, name, note)| {
                    GuestbookEntryStamped {time_stamp, name, note}
                }
            );

            let guestbook_table = match query_res {
                Ok(table) => {table},
                Err(_) => {
                    println!("Couldn't get data from database, likely due to malformed query.");
                    return send_500_json();
                }
            };

            // serialize table into string, with main keys being row numbers
            // ID values are irrelevant here, so an incrementing index will do just as well
            let mut table_string = format!("[{}", guestbook_table
                .iter()
                .map(|gbs| {
                    format!("{},", serde_json::to_string(gbs).unwrap())
                })
                .collect::<Vec<String>>()
                .concat()
            );
            table_string.pop();   // remove terminal comma, on every line from the the last statement
            table_string.push_str("]");

            with_status(json::<String>(&table_string), StatusCode::OK)
        }

        Err(_) => {
            println!("Couldn't connect to database. Make sure it's running!");
            send_500_json()
        }
    }
}


pub async fn update_guestbook(form_entry: GuestbookEntry) -> axum::response::Html<String> {

    // db connection setup
    let Some(buf_pool) = get_db_conn() 
        else { 
            return send_500_html(); 
        };
    
    // converting in this scope so that the name is in scope
    // for printing name on entry to server console (for fun)
    let entry_name = if form_entry.name.is_empty() {
        "(anonymous)".to_string()
    } else {
        form_entry.name
    };

    if let Ok(mut conn) = buf_pool.get_conn() {
        let insert_res: Result<Vec<Row>, Error> = conn.exec(
            r"INSERT INTO guestbook (dateSubmitted, guestName, guestNote)
                    VALUES (UTC_TIMESTAMP(), :name, :note)",
            params! {
                "name" => &entry_name, 
                "note" => form_entry.note
            }
        );

        match insert_res {
            Ok(_) => {
                println!("New entry in the guestbook from {}!", entry_name);
                return axum::response::Html(String::from(
                        "<!DOCTYPE html>\n\
                        <html><head>\n\
                        <title>Entry Received!</title>\n\
                        </head>\n\
                        <p>Thanks for leaving a note on my website!</p>\n\
                        </html>"
                ));
            },
            Err(e) => {
                println!("Couldn't get data from database, likely due to internally \
                    malformed query. The error is: {}", e);
                return send_500_html();
            }
        };
    } else {
        println!("Couldn't connect to database. Make sure it's running!");
        send_500_html()
    }
}


// Updates the list of timestamps (corresponding to home page hits),
// then returns the total number of hits as a JSON response.
// This is because, on this website, hits are defined by how many
// GET requests there are to /hits.
pub async fn update_hits() -> impl Reply {
    
    let buf_pool = match get_db_conn() {
        Some(bp) => { bp },
        None => { return send_500_json(); }
    };

    if let Ok(mut conn) = buf_pool.get_conn() {
        
        let tx_options = TxOpts::default()
            .set_isolation_level(Some(
                IsolationLevel::Serializable
            ));

        // unwrapping b/c the outer if-let would have caught any 
        // conn issues, and I know that the transaction options are
        // allowed for the database
        let mut tx = conn
            .start_transaction(tx_options)
            .unwrap();   
        
        // INSERT statements return a string with the number of rows effected, 
        // warnings, and duplicates.
        // In this context it doesn't matter, except for type annotations
        if let Ok(_) = tx.exec_first::<String, &str, Params>(
            r"INSERT INTO hitsLog (hitTime) VALUES (UTC_TIMESTAMP());",
            Params::Empty
        ) {
            tx.commit().unwrap(); // as before, conn issues would have been handled previously
        }
        else {
            println!("Couldn't get data from database, likely due to internally malformed query.");
            return send_500_json();
        };
        
        if let Ok(Some(hits_count)) = conn.query_first::<u32, &str>(
            r"SELECT COUNT(*) AS hits_count FROM hitsLog"
        ) {
            with_status(json(&format!(
                "{{\"hits_count\": {}}}", hits_count
            )),
            StatusCode::OK)
        } else {
            println!("Couldn't get data from database, \
                likely due to internally malformed query.");
            send_500_json()
        }
        
    } else {
        println!("Couldn't get data from database, \
            likely due to internally malformed query.");
        return send_500_json();
    }
}