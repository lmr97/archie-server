use std::net::SocketAddr;
use std::fs::OpenOptions;
use axum::{
    routing::{get, get_service, post}, 
    Router
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{
    services::ServeDir,
    services::ServeFile,
};
use tracing::info;

mod archie_utils;
mod err_handling;
mod db_io;

#[tokio::main]
async fn main() {

    let log_file = OpenOptions::new()
        .append(true)
        .open("/var/log/archie-server.log")
        .unwrap();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(log_file)
        .init();

    info!("Loading certificates and keys...");
    let (cert, pks) = archie_utils::get_auth_paths();
    let auth_config = RustlsConfig::from_pem_file(cert, pks)
        .await
        .unwrap();                  // I don't want to serve the webpage without TLS, so crash okay
    info!("Authorization loaded!");

    info!("Defining routes...");
    //let guestbook = Router::new().nest("/", );
    let homepage = ServeFile::new(format!("{}/home.html", archie_utils::LOCAL_ROOT));
    let static_dir = ServeDir::new(format!("{}/static", archie_utils::LOCAL_ROOT));

    let guestbook_page = ServeFile::new(format!("{}/guestbook.html", archie_utils::LOCAL_ROOT));
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
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 443));
    info!("Serving on {:?}!", addr);

    axum_server::bind_rustls(addr, auth_config)
        .serve(routes.into_make_service())
        .await
        .unwrap();                  // should cause crash; fatal error to server
}
