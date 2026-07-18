use std::sync::Arc;

use tracing_subscriber::{filter::LevelFilter, layer::SubscriberExt, util::SubscriberInitExt};
use wtimer_lib::{config, server};

use tracing::info;
use wtimer_lib::infra::redb;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load();

    setup_logger(cfg.log_level);
    info!(config = ?cfg, "wtimer started");

    let db = Arc::new(redb::connect(cfg.db)?);

    let range_repo = redb::range::Repository::new(db)?;

    let app = server::setup(range_repo);

    let listener = tokio::net::TcpListener::bind((cfg.address, cfg.port)).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;

    Ok(())
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
