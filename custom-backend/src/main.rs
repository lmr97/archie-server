use std::collections::HashMap;
use std::{
    convert::Infallible,
    env, 
    error::Error as StdError,
    fs::OpenOptions,
    io::Write
};
use chrono::prelude::*;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};
use serde_derive::{Deserialize, Serialize};

static LOCAL_ROOT: &str = "/home/martinr/archie-server";

#[derive(Debug, Deserialize, Serialize)] 
struct GuestbookEntry {
    name: String,
    note: String
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {

    //let (cert, pks) = get_auth_paths();
    
    let home = warp::path::end()
        .and(warp::fs::file(format!("{}/home.html", LOCAL_ROOT)));
    
    let static_content = warp::path("static")
        .and(warp::fs::dir(format!("{}/static/", LOCAL_ROOT)))
        .with(warp::compression::gzip());

    let node_modules = warp::path("node_modules")
        .and(warp::fs::dir(format!("{}/node_modules/", LOCAL_ROOT)))
        .with(warp::compression::gzip());

    let guestbook_page = warp::path("guestbook")
        .and(warp::fs::file(format!("{}/guestbook.html", LOCAL_ROOT)));

    let guestbook_entry = warp::path("guestbook-entries")
        .and(warp::body::json::<GuestbookEntry>())
        .then(update_guestbook);

    let guestbook_data = warp::path("guestbook.csv")
        .and(warp::fs::file(format!("{}/guestbook.csv", LOCAL_ROOT)));
        

    let routes = 
        warp::get().and(
            home
            .or(static_content)
            .or(guestbook_page)
            .or(guestbook_data)
            .or(node_modules)).or(
        warp::post().and(
                guestbook_entry
            ))
        .recover(handle_rejection);

    warp::serve(routes)
        //.tls()
        //.cert_path(cert)
        //.key_path(pks)
        .run(([127,0,0,1], 8080))
        .await;

    Ok(())
}


fn get_auth_paths() -> (String, String) {

    // load in certs from environment filepaths
    let cert_file = env::var_os("CRT_FILE")
        .expect("Certificates filepath variable not found in environment.")
        .into_string()
        .unwrap();
    let private_key_file = env::var_os("PK_FILE")
        .expect("Private keys filepath variable not found in environment.")
        .into_string()
        .unwrap();

    (cert_file, private_key_file)
}


async fn update_guestbook(form_entry: GuestbookEntry) -> impl Reply {
    
    let entry_name = if form_entry.name.is_empty() {
        "anonymous".to_string()
    } else {
        form_entry.name
    };

    let gb_file_res = OpenOptions::new()
        .append(true)
        .open(format!("{}/guestbook.csv", LOCAL_ROOT));


    match gb_file_res {
        
        Ok(mut guestbook_file) => { 
            
            let curr_datetime = Local::now()
                .format("%A %I:%M%p, %B %-d %Y")
                .to_string();

            let entry_line = format!(
                "\"{}\",\"{}\",\"{}\"",  // wrap fields in quotes
                curr_datetime,  
                form_entry.note,
                entry_name,
            );
            
            if let Err(e) = writeln!(
                guestbook_file, "{entry_line}"
            ) {
                eprintln!("ERROR: Found file, but couldn't write to it: {}", e);
            }
        },
        Err(_) => {
            println!("ERROR: Could not find guestbook file at {}/guestbook.csv", 
                LOCAL_ROOT
            );
        }
    };
    
    println!("New entry in the guestbook from {}!", entry_name);
    warp::reply::with_status(
        warp::reply::html(
            String::from("\
                <!DOCTYPE html>\n\
                <html><head>\n\
                <title>Entry Received!</title>\n\
                </head>\n\
                <p>Thanks for leaving a note on my website!</p>\n\
                </html>"
            )
        ), StatusCode::OK
    )
}


// very simple right now, to match the server
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    
    // easily extensible way to handle errors with custom HTML pages 
    // simply make a new error page with the error code as the filename,
    // in ./static/errors with a .html extension,
    // and update the `supported_err_codes` array below.
    let supported_err_codes = ["404", "500"];
    let mut err_html: HashMap<&str, String> = HashMap::new();

    for code in supported_err_codes {
        let code_res = std::fs::read_to_string(
            format!(
                "{}/static/errors/{}.html", 
                LOCAL_ROOT,
                code
        ));

        let html_string= match code_res {
            Ok(html_string) => { html_string },
            Err(_) => {
                println!("ERROR: {}.html not found at usual location, sending unpretty version.", code);
                format!("\
                    <!DOCTYPE html>\n\
                    <html><head>\n\
                    <title>{code} Error</title>\n\
                    </head>\n\
                    {code} Error\n\
                    </html>"
                )
            }
        };

        err_html.insert(code, html_string);
    }

    println!(
        "{}: Encountered error: {:?}", 
        Local::now()
            .format("%a %d %b %Y, %I:%M%p")
            .to_string(), 
        err
    );
    
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            warp::reply::html(err_html["404"].clone()), 
            StatusCode::NOT_FOUND
        ))
    } else {
        Ok(warp::reply::with_status(
            warp::reply::html(err_html["500"].clone()), 
            StatusCode::INTERNAL_SERVER_ERROR
        ))
    }
}