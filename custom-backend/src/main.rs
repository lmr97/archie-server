use std::net::SocketAddr;
use axum::{
    routing::{get, get_service, post}, 
    Router
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{
    services::ServeDir,
    services::ServeFile,
    //trace::TraceLayer,
};

mod utils;
mod err_handling;
mod db_io;

#[tokio::main]
async fn main() {

    println!("Loading certificates and keys...");
    let (cert, pks) = utils::get_auth_paths();
    let auth_config = RustlsConfig::from_pem_file(cert, pks)
        .await
        .unwrap();
    println!("Authorization loaded!");

    println!("Defining routes...");
    //let guestbook = Router::new().nest("/", );
    let homepage = ServeFile::new(format!("{}/home.html", utils::LOCAL_ROOT));
    let static_dir = ServeDir::new(format!("{}/static", utils::LOCAL_ROOT));

    let guestbook_page = ServeFile::new(format!("{}/guestbook.html", utils::LOCAL_ROOT));
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
    let addr = SocketAddr::from(([0, 0, 0, 0], 443));
    axum_server::bind_rustls(addr, auth_config)
        .serve(routes.into_make_service())
        .await
        .unwrap();
}
