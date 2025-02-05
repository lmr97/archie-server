use std::env; 
use chrono::prelude::*;
use warp::http::StatusCode;
use warp::reply::{
    with_status,
    Html, html, 
    Json, json, 
    WithStatus
};
use warp::Reply;
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

// gets URL from the environment to preserve security,
// since it contains a plain-text password
fn get_db_url() -> String {
    env::var_os("DB_URL")
        .expect("Database URL variable not found in environment.")
        .into_string()
        .unwrap()
}


fn send_500_html() -> WithStatus<Html<String>> {
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

    with_status(
        html(html_string.clone()), 
        StatusCode::INTERNAL_SERVER_ERROR)
}

fn send_500_json() -> WithStatus<Json> {
    with_status(json::<String>(
        &String::from(
            "{\"status\": \"error\", \
            \"message\": \"Likely a database communication error\", \
            \"code\": 500}"
        )
    ), StatusCode::INTERNAL_SERVER_ERROR)
}

// it is a simpler, albeit slower, design to make the connection every
// time the function is called
pub async fn get_guestbook() -> WithStatus<Json> {
    let url = get_db_url();
    let opts_res = Opts::from_url(&url);

    let buf_pool = match opts_res {
        Ok(opts) => {Pool::new(opts).unwrap()},  // unwrap() b/c infallible
        Err(_) => {
            println!("Couldn't find database at provided URL: {}.", url);
            return send_500_json();
        }
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


pub async fn update_guestbook(form_entry: GuestbookEntry) -> impl Reply {

    // db connection setup
    let url = get_db_url();
    let opts_res = Opts::from_url(&url);

    let buf_pool = match opts_res {
        Ok(opts) => {Pool::new(opts).unwrap()},  // unwrap() b/c infallible
        Err(_) => {
            println!("Couldn't find database at provided URL: {}.", url);
            return send_500_html();
        }
    };

    // doing this in this scope so that the name is in scope
    // for printing name on entry to server console (for fun)
    let entry_name = if form_entry.name.is_empty() {
        "(anonymous)".to_string()
    } else {
        form_entry.name
    };

    let conn_res = buf_pool.get_conn();
    match conn_res {
        Ok(mut conn) => {
            
            let curr_datetime = Utc::now().naive_local();

            let insert_res: Result<Vec<Row>, Error> = conn.exec(
                r"INSERT INTO guestbook (dateSubmitted, guestName, guestNote)
                        VALUES (:now, :name, :note)",
                params! {
                    "now"  => curr_datetime,
                    "name" => &entry_name, 
                    "note" => form_entry.note
                }
            );

            match insert_res {
                Ok(_) => {
                    println!("New entry in the guestbook from {}!", entry_name);
                    return warp::reply::with_status(
                        html(
                            String::from("\
                                <!DOCTYPE html>\n\
                                <html><head>\n\
                                <title>Entry Received!</title>\n\
                                </head>\n\
                                <p>Thanks for leaving a note on my website!</p>\n\
                                </html>"
                            )
                        ), StatusCode::OK
                    );
                },
                Err(_) => {
                    println!("Couldn't get data from database, likely due to internally malformed query.");
                    return send_500_html();
                }
            };
        }

        Err(_) => {
            println!("Couldn't connect to database. Make sure it's running!");
            send_500_html()
        }

    }
}