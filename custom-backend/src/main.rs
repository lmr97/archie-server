use std::{
    fs::OpenOptions,
    net::SocketAddr,
    process::exit,
    str::FromStr
};
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
use tracing_subscriber::EnvFilter;

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

    let arg1 = std::env::args().nth(1);

    let no_tls = match arg1 {
        Some(arg) => {
            match arg.as_str() {
                "--no-tls"      => true,
                "--help" | "-h" => {
                    print_help();
                    exit(0);
                }
                other => {
                    print_help();
                    panic!("Option \"{other}\" is not recognized.");
                }
            }
        },
        None => false
    };

    let log_file = OpenOptions::new()
        .append(true)
        .open(get_env_var("SERVER_LOG"))
        .unwrap();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(log_file)
        .with_line_number(true)
        .init();

    info!("\n\n\t\t////// Hi there! I'm Archie. Let me get ready for you... //////\n");

    info!("Defining routes...");
    let server_root = get_env_var("SERVER_ROOT");

    let homepage  = ServeFile::new(format!("{}/pages/home.html", server_root));
    let static_dir = ServeDir::new(format!("{}/static", server_root));

    let guestbook_page = ServeFile::new(format!("{}/pages/guestbook.html", server_root));
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

    if no_tls {
        info!("Serving on {:?}!", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, routes).await.unwrap();
    } else {
        
        info!("Loading certificates and keys...");
        let (cert, pks) = archie_utils::get_auth_paths();
        let auth_config = RustlsConfig::from_pem_file(cert, pks)
            .await
            .unwrap();                  // I don't want to serve the webpage without TLS, so crash okay
        info!("Authorization loaded!");
    
        info!("Serving on {:?}!", addr);
        axum_server::bind_rustls(addr, auth_config)
            .serve(routes.into_make_service())
            .await
            .unwrap();                  // should cause crash; fatal error to server
    }
}

fn print_help() {
    println!("Usage:  custom-backend [OPTION]\n");
    println!("The executable that runs the server.\n");
    println!("Options:");
    println!("    --no-tls     Run without TLS. Axum doesn't serve files properly");
    println!("                 on localhost with TLS, so this is good for demo purposes.");
    println!("    --help, -h   Print this help message and quit.\n");
}