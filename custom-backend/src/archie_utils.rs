use std::collections::HashMap;
use std::{convert::Infallible, env};
use chrono::prelude::*;
use warp::http::StatusCode;
use warp::reply::{with_status, html};
use warp::{Rejection, Reply};
use mysql::*;

pub static LOCAL_ROOT: &str = "/home/martin/archie-server";

pub fn get_auth_paths() -> (String, String) {

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


// very simple right now, to match the server
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    
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
                    <p>{code} Error</p>\n\
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
        Ok(with_status(
            html(err_html["404"].clone()), 
            StatusCode::NOT_FOUND
        ))
    } else {
        Ok(with_status(
            html(err_html["500"].clone()), 
            StatusCode::INTERNAL_SERVER_ERROR
        ))
    }
}