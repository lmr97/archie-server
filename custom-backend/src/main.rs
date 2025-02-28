use std::{net::SocketAddr, str::FromStr};
use std::fs::OpenOptions;
use archie_utils::get_env_var;
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

// the frequent usage of unwrap() in main() here is because
// any panics that happen in main() will cause the server to 
// crash on startup, and not during subsequent runtime (I made 
// sure to purge functions that could be called after server 
// startup of any calls to unwrap()).
#[tokio::main]
async fn main() {

    let log_file = OpenOptions::new()
        .append(true)
        .open(get_env_var("SERVER_LOG"))
        .unwrap();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(log_file)
        .init();

    info!("\n\n\t\t////// Hi there! I'm Archie. Let me get ready for you... //////\n");
    info!("Loading certificates and keys...");
    let (cert, pks) = archie_utils::get_auth_paths();
    let auth_config = RustlsConfig::from_pem_file(cert, pks)
        .await
        .unwrap();                  // I don't want to serve the webpage without TLS, so crash okay
    info!("Authorization loaded!");

    info!("Defining routes...");
    let server_root = get_env_var("SERVER_ROOT");

    let homepage  = ServeFile::new(format!("{}/home.html", server_root));
    let static_dir = ServeDir::new(format!("{}/static", server_root));

    let guestbook_page = ServeFile::new(format!("{}/guestbook.html", server_root));
    let guestbook_entries: Router<()> = Router::new()
        .route("/", post(db_io::update_guestbook))
        .route("/", get(db_io::get_guestbook));
    let guestbook_routes = Router::new()
        .route_service("/", get_service(guestbook_page))
        .nest("/entries", guestbook_entries);

    let routes = Router::new()
        .route("/", get_service(homepage))
        .nest("/guestbook", guestbook_routes)
        .route("/hits", post(db_io::log_hit))
        .route("/hits", get(db_io::get_hit_count))
        .nest_service("/static", get_service(static_dir));
        
    let addr = SocketAddr::from_str(&get_env_var("SERVER_SOCKET")).unwrap();

    info!("Serving on {:?}!", addr);
    axum_server::bind_rustls(addr, auth_config)
        .serve(routes.into_make_service())
        .await
        .unwrap();                  // should cause crash; fatal error to server
}
