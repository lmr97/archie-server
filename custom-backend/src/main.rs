use std::{
    env, 
    error::Error as StdError,
    convert::Infallible
};
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

static LOCAL_ROOT: &str = "/home/martin/archie-server";

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {

    let (cert, pks) = get_auth_paths();
    
    let home = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file(format!("{}/home.html", LOCAL_ROOT)));

    let image = warp::get()
        .and(warp::path("images"))
        .and(warp::fs::dir(format!("{}/images/", LOCAL_ROOT)))
        .with(warp::compression::gzip());

    let routes = home.or(image).recover(handle_rejection);
    warp::serve(routes)
        .tls()
        .cert_path(cert)
        .key_path(pks)
        .run(([0,0,0,0], 443))
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

// very simple right now, to match the server
async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    
    let err_404 = warp::reply::html(
        std::fs::read_to_string(format!("{}/errors/404.html", LOCAL_ROOT))
            .expect("File not found.")
    );
    let err_500 = warp::reply::html(
        std::fs::read_to_string(format!("{}/errors/404.html", LOCAL_ROOT))
        .expect("File not found.")
    );
    
    if err.is_not_found() {
        Ok(warp::reply::with_status(err_404, StatusCode::NOT_FOUND))
    } else {
        Ok(warp::reply::with_status(err_500, StatusCode::INTERNAL_SERVER_ERROR))
    }
}