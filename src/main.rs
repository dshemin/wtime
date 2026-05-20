mod config;
mod handler;

use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), impl std::error::Error> {
    let cfg = config::load();

    setup_logger(cfg.log_level);

    let app = Router::new()
        .route("/", get(handler::index_handler))
        .route("/{*file}", get(handler::static_handler))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(TraceLayer::new_for_http());

    info!(config = ?cfg, "wtimer started");
    let listener = tokio::net::TcpListener::bind((cfg.address, cfg.port)).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
}

fn setup_logger(log_level: tracing::Level) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true),
        )
        .with(LevelFilter::from_level(log_level))
        .init();
}

async fn shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install terminate handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    };

    info!("wtimer stopped");
}
