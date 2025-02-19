use std::error::Error as StdError;
use axum::{
    routing::{
        get, 
        get_service, 
        post
    }, 
    Router
};

use tower_http::{
    services::ServeDir,
    services::ServeFile,
    //trace::TraceLayer,
};

mod archie_utils;
mod db_io;

const LOCAL_TEST: &str = "/home/martinr/archie-server";
#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {

    println!("Loading certificates and keys...");
    //let (cert, pks) = archie_utils::get_auth_paths();
    println!("Authorization found!");

    println!("Defining routes...");
    //let guestbook = Router::new().nest("/", );
    let homepage = ServeFile::new(format!("{}/home.html", LOCAL_TEST));
    let static_dir = ServeDir::new(format!("{}/static", LOCAL_TEST));

    let guestbook_page = ServeFile::new(format!("{}/guestbook.html", LOCAL_TEST));
    let guestbook_entries: Router<()> = Router::new()
        .route("/", post(db_io::update_guestbook))
        .route("/", get(db_io::get_guestbook));
    let guestbook_routes = Router::new()
        .route_service("/", get_service(guestbook_page))
        .nest("/entries", guestbook_entries);

    let routes = Router::new()
        .route("/", get_service(homepage))
        .nest("/guestbook", guestbook_routes)
        .route("/hits", get(db_io::update_hits))
        .nest_service("/static", get_service(static_dir));
    
    println!("Serving!");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, routes).await.unwrap();

    Ok(())
}