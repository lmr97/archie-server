use std::{
    net::SocketAddr,
    str::FromStr
};
use axum::{
    routing::{get, post}, 
    Router
};
use axum_server::tls_rustls::RustlsConfig;
use tower_http::{
    services::ServeDir,
    compression::CompressionLayer
};
use tracing::info;
use custom_backend::{
    srv_io::{vite_io, db_io, lb_app_io},
    utils::init_utils::*,
};



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


    /* Guestbook */
    let guestbook_entries: Router<()> = Router::new()
        .route("/", post(db_io::update_guestbook))
        .route("/", get(db_io::get_guestbook));
    let guestbook_app = Router::new()
        .route("/", get(vite_io::guestbook_page))
        .nest("/entries", guestbook_entries);


    /* Letterboxd List Converter */
    let converter = Router::new().route("/conv", get(lb_app_io::convert_lb_list));
    let lb_app = Router::new()
        .route("/", get(vite_io::lb_app_page))
        .nest("/conv", converter);

    
    /* All routes */
    let routes = Router::new()
        .route("/", get(vite_io::homepage))
        .nest_service("/assets", ServeDir::new(format!("{}/dist/assets", server_root)))
        .route("/hits", post(db_io::log_hit))
        .route("/hits", get(db_io::get_hit_count))
        .nest("/guestbook", guestbook_app)       // `Router`s must be nested with other routers
        .nest("/lb-list-conv", lb_app)
        .layer(CompressionLayer::new());         // compress all responses

    // Start Vite dev server (debug only)
    let _guard = vite_io::VitePage::start_dev_server(true);  
    

    let addr = SocketAddr::from_str(&get_env_var("SERVER_SOCKET").unwrap()).unwrap();

    if !use_tls {

        info!("Serving on {:?}! (no TLS)", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, routes).await?;

    } else {
        
        info!("Loading certificates and keys...");
        let (cert, pks) = get_auth_paths();
        let auth_config = RustlsConfig::from_pem_file(cert, pks)
            .await?;
        info!("Authorization loaded!");
    
        info!("Serving securely on {:?}!", addr);
        axum_server::bind_rustls(addr, auth_config)
            .serve(routes.into_make_service())
            .await?;
    }

    Ok(())
}
