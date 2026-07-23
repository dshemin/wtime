use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::domain::range::Repository;
use crate::handler;

pub fn setup<T: Repository + 'static>(range_repo: T) -> Router {
    let range_repo = Arc::new(range_repo);

    let api = Router::new()
        .route("/ranges", get(handler::ranges::list))
        .route("/ranges", post(handler::ranges::create))
        .route("/ranges", put(handler::ranges::update))
        .route("/ranges/{id}", delete(handler::ranges::delete))
        .with_state(range_repo);

    let tout_layer =
        TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10));

    let trace_layer = TraceLayer::new_for_http();

    let app = Router::new()
        .route("/", get(handler::index_handler))
        .route("/{*file}", get(handler::static_handler))
        .nest("/api/v1", api)
        .layer(tout_layer)
        .layer(trace_layer);

    #[cfg(debug_assertions)]
    let app = {
        use axum::http::{Method, header};
        use tower_http::cors::{Any, CorsLayer};

        let cors = CorsLayer::new()
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
            .allow_methods([
                Method::HEAD,
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
            ])
            .allow_origin(Any);

        app.layer(cors)
    };

    app
}
