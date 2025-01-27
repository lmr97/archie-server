use std::{env, error::Error as StdError};


use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {

    let (cert, pks) = get_auth_paths();
    
    let home = warp::get()
        .and(warp::path("/"))
        .and(warp::fs::file("../home.html"));

    let image = warp::path("images")
        .and(warp::fs::dir("../images/"));

    let routes = home.or(image);
    warp::serve(routes)
        .tls()
        .cert_path(cert)
        .key_path(pks)
        .run(([127,0,0,1], 443))
        .await;

    Ok(())
}


fn get_auth_paths() -> (String, String) {

    // Modified version of simpleserver.rs example from Rustls docs
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