use std::{
    net::SocketAddr,
    str::FromStr
};
use custom_backend::{
    srv_io::{db_io, lb_app_io},
    utils::init_utils::*,
};
use axum::{
    routing::{get, post}, 
    Router
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{
    services::ServeDir,
    services::ServeFile,
    compression::CompressionLayer
};
use tracing::info;

// The only panicking unwraps are here in main(), since, if there are 
// any problems during startup, I want the server to crash and show me what 
// went wrong. You may see unwrap() in the rest of the source code, 
// but these are guaranteed not to panic, either given the Result content,
// or a restriction on the possible values of the calling function's input.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // stdout can be suppressed when env var PRE_LOG=1
    let prelog = match get_env_var("PRE_LOG") {
        Ok(prelog_val) => {
            if prelog_val == "0" { false }
            else { 
                println!("To suppress pre-log output to stdout, set PRE_LOG=1.");
                true 
            }
        },
        Err(_) => false
    };

    if prelog { println!("[ PRE-LOG ]: Loading log file path from environment..."); }
    
    let log_file_path = get_env_var("SERVER_LOG")?;
    build_logger(log_file_path, prelog)?.init();

    if prelog { println!("[ PRE-LOG ]: Logger initialized!"); }
    
    

    info!("\n\n\t\t////// Hi there! I'm Archie. Let me get ready for you... //////\n");

    info!("Reading initialization options...");

    let use_tls = match process_cli_args()? {
        RunMode::PrintHelp => return Ok(()),
        RunMode::NoTls => false,
        RunMode::Tls => true
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
        .route("/conv", get(lb_app_io::convert_lb_list));

    
    /* All routes */
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


    if !use_tls {

        info!("Serving on {:?}! (no TLS)", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, routes).await?;

    } else {
        
        info!("Loading certificates and keys...");
        let (cert, pks) = get_auth_paths();
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
