use tokio::signal;
use tracing::info;
use axum_server::Handle;

// Adapted from the Axum examples on Github: 
// https://github.com/tokio-rs/axum/blob/main/examples/tls-graceful-shutdown/src/main.rs
pub async fn on_signal(handle: Option<Handle>) {

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    let sig_recved: &'static str;

    tokio::select! {
        _ = ctrl_c    => { sig_recved = "SIGINT";  },
        _ = terminate => { sig_recved = "SIGTERM"; },
    }

    if let Some(svr_handle) = handle {
        svr_handle.graceful_shutdown(None);
    }

    // sending to both stdout and the log file for redundancy
    println!("\nI just got a {} signal; shutting down now. Bye!", sig_recved);
    info!("I just got a {} signal; shutting down now. Bye!", sig_recved);
}