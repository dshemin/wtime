pub mod ranges;
mod r#static;

use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;
pub use r#static::*;

#[derive(Serialize)]
pub struct ErrorResponse {
    message: String,
    #[serde(with = "status_code_serde")]
    code: StatusCode,
}

pub type Result<T> = std::result::Result<Json<T>, ErrorResponse>;

macro_rules! err_resp_new {
    ($name:ident, $code:expr) => {
        pub fn $name<T: Into<String>>(msg: T) -> Self {
            Self {
                message: msg.into(),
                code: $code,
            }
        }
    };
}

impl ErrorResponse {
    err_resp_new!(bad_request, StatusCode::BAD_REQUEST);
    err_resp_new!(internal, StatusCode::INTERNAL_SERVER_ERROR);
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (self.code, Json(self)).into_response()
    }
}

mod status_code_serde {
    use axum::http::StatusCode;
    use serde::Serializer;

    pub fn serialize<S>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serializes the status code as a u16 integer (e.g., 200)
        serializer.serialize_u16(status.as_u16())
    }
}
