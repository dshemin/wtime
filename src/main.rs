mod handler;

use axum::{Router, routing::get};

#[tokio::main]
async fn main() -> Result<(), impl std::error::Error> {
    let app = Router::new()
        .route("/", get(handler::index_handler))
        .route("/{*file}", get(handler::static_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await
}
