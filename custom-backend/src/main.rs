use std::{
    net::SocketAddr,
    str::FromStr
};
use axum::{
    routing::{get, post}, 
    Router
};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use tower_http::{compression::CompressionLayer};
use tracing::info;

use custom_backend::{
    srv_io::{vite_get, db_io, lb_app_io},
    utils::{init_utils::*, shutdown},
};

#[derive(vite_rs::Embed)]
#[root = "../"] 
#[dev_server_port = 5173]
#[allow(dead_code, reason = "AssetHandle used to run Vite dev server in debug builds")]
struct AssetHandle;

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
        RunMode::NoTls     => false,
        RunMode::Tls       => true
    };



    info!("Defining routes...");

    let api = Router::new()
        .route("/hits", get(db_io::get_hit_count))
        .route("/hits", post(db_io::log_hit))
        .route("/guestbook/entries", get(db_io::get_guestbook))
        .route("/guestbook/entries", post(db_io::update_guestbook))
        .route("/lb-list-conv/conv", get(lb_app_io::convert_lb_list));
    
    let routes = Router::new()
        .route("/", get(vite_get::serve_statics))
        .route("/{*path}", get(vite_get::serve_statics))
        .merge(api)
        .layer(CompressionLayer::new());         // compress all responses


    // Start Vite dev server (on debug only).
    // 
    // NOTE TO SELF: To indicate to Vite that it's merely being 
    // puppeted by a Rust program in debug mode (and to not run 
    // the mock backend I wrote to emulate this Rust server), set 
    // the env var VITE_SVR_MODE to "rust" (or anything so 
    // long as it's not "native")

    #[cfg(debug_assertions)]
    let _guard = AssetHandle::start_dev_server(true);  
    


    let addr = SocketAddr::from_str(&get_env_var("SERVER_SOCKET").unwrap()).unwrap();
    let svr_handle = Handle::new(); // allows other threads access to the thread running the server
    
    if !use_tls {

        info!("Serving on {:?}! (no TLS)", addr);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, routes)
            .with_graceful_shutdown(shutdown::on_signal(None))
            .await?;

    } else {
        
        info!("Loading certificates and keys...");
        let (cert, pks) = get_auth_paths();
        let auth_config = RustlsConfig::from_pem_file(cert, pks)
            .await?;
        info!("Authorization loaded!");
    
        // Spawn a task to catch signals and shut down server; 
        // they won't be caught in this thread.
        tokio::spawn(shutdown::on_signal(Some(svr_handle.clone())));

        info!("Serving securely on {:?}!", addr);
        axum_server::bind_rustls(addr, auth_config)
            .handle(svr_handle)
            .serve(routes.into_make_service())
            .await?;
    }

    Ok(())
}
