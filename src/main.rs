use axum::{
    Json, Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "frontend/build"]
struct Asset;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/{*file}", get(static_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await;
}

async fn index_handler() -> impl IntoResponse {
    static_handler(Path("index.html".to_string())).await
}

async fn static_handler(Path(path): Path<String>) -> impl IntoResponse {
    StaticFile(path)
}

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();

        match Asset::get(path.as_str()) {
            Some(content) => {
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
            }
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
        }
    }
}
