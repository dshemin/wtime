mod handler;

use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::timeout::TimeoutLayer;

#[tokio::main]
async fn main() -> Result<(), impl std::error::Error> {
    let app = Router::new()
        .route("/", get(handler::index_handler))
        .route("/{*file}", get(handler::static_handler))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
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
    }
}
