use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::{get, post};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::domain::range::Repository;
use crate::handler;

pub fn setup<T: Repository + 'static>(range_repo: T) -> Router {
    let range_repo = Arc::new(range_repo);

    let api = Router::new()
        .route("/ranges/{date}", get(handler::ranges::list))
        .route("/ranges", post(handler::ranges::create))
        .with_state(range_repo);

    Router::new()
        .route("/", get(handler::index_handler))
        .route("/{*file}", get(handler::static_handler))
        .nest("/api/v1", api)
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(10),
        ))
        .layer(TraceLayer::new_for_http())
}
