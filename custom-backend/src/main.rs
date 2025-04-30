use std::{
    io::{Error, ErrorKind},
    fs::OpenOptions,
    net::SocketAddr,
    str::FromStr
};
use archie_utils::get_env_var;
use axum::{
    routing::{get, post, any}, 
    Router
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{
    services::ServeDir,
    services::ServeFile,
    compression::CompressionLayer
};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod archie_utils;
mod err_handling;
mod db_io;
mod lb_app_io;

// The only panicking unwraps are here in main(), since, if there are 
// any problems during startup, I want the server to crash and show me what 
// went wrong. You may see unwrap() in the rest of the source code, 
// but these are guaranteed not to panic, either given the Result content,
// or a restriction on the possible values of the calling function's input.

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    // println!() has to be used here, because the logger is not yet initialized
    println!("[ PRE-LOG ]: Loading log file...");
    let log_file = OpenOptions::new()
        .append(true)
        .open(get_env_var("SERVER_LOG").unwrap())
        .unwrap();
    println!("[ PRE-LOG ]: Log file loaded!");

    println!("[ PRE-LOG ]: Initializing logger...");
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(log_file)
        .init();
    println!("[ PRE-LOG ]: Logger initialized!");


    info!("\n\n\t\t////// Hi there! I'm Archie. Let me get ready for you... //////\n");

    // process CLI args
    let arg1 = std::env::args().nth(1);

    let no_tls = match arg1 {
        Some(arg) => {
            match arg.as_str() {
                "--no-tls"      => true,
                "--help" | "-h" => {
                    print_help();
                    return Ok(());
                }
                other => {
                    print_help();
                    return Err(
                        Error::new(
                            ErrorKind::InvalidInput,
                            format!("Option \"{other}\" is not recognized.")
                        )
                    );
                }
            }
        },
        None => false
    };

    info!("Defining routes...");
    let server_root = get_env_var("SERVER_ROOT").unwrap();

    /* Homepage/general */
    let homepage  = ServeFile::new(format!("{}/pages/home.html", server_root));
    let static_dir = ServeDir::new(format!("{}/static", server_root));
    let node_mods  = ServeDir::new(format!("{}/node_modules", server_root));

    /* Guestbook */
    let guestbook_page = ServeFile::new(format!("{}/pages/guestbook.html", server_root));
    let guestbook_entries: Router<()> = Router::new()
        .route("/", post(db_io::update_guestbook))
        .route("/", get(db_io::get_guestbook));
    let guestbook_app = Router::new()
        .route_service("/", guestbook_page)
        .nest("/entries", guestbook_entries);

    /* Letterbocd List Converter */
    let lb_app_page = ServeFile::new(format!("{}/pages/lb-list-app.html", server_root));
    let lb_app = Router::new()
        .route_service("/", lb_app_page)
        .route("/conv", any(lb_app_io::convert_lb_list));

    let routes = Router::new()
        .route_service("/", homepage)
        .nest_service("/static", static_dir)
        .nest_service("/node_modules", node_mods)
        .route("/hits", post(db_io::log_hit))
        .route("/hits", get(db_io::get_hit_count))
        .nest("/guestbook", guestbook_app)       // `Router`s must be nested with other routers
        .nest("/lb-list-conv", lb_app)
        .layer(CompressionLayer::new());         // compress all responses

    let addr = SocketAddr::from_str(&get_env_var("SERVER_SOCKET").unwrap()).unwrap();

    if no_tls {

        info!("Serving on {:?}! (no TLS)", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, routes).await?;

    } else {
        
        info!("Loading certificates and keys...");
        let (cert, pks) = archie_utils::get_auth_paths();
        let auth_config = RustlsConfig::from_pem_file(cert, pks)
            .await?;                  // I don't want to serve the webpage without TLS, so crash okay
        info!("Authorization loaded!");
    
        info!("Serving securely on {:?}!", addr);
        axum_server::bind_rustls(addr, auth_config)
            .serve(routes.into_make_service())
            .await?;                  // should cause crash; fatal error to server
    }

    Ok(())
}

fn print_help() {
    println!("Usage:  custom-backend [OPTION]\n");
    println!("The executable that runs the server.\n");
    println!("Options:");
    println!("    --no-tls     Run without TLS. Axum doesn't serve files properly");
    println!("                 on localhost with TLS, so this is good for demo purposes.");
    println!("    --help, -h   Print this help message and quit.\n");
}