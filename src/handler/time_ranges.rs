use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::NaiveDate;
use tracing::error;

use crate::storage::{TimeRange, TimeRangeStorage};

pub async fn list(
    State(strg): State<TimeRangeStorage>,
    Path(date): Path<NaiveDate>,
) -> ListResult<TimeRange> {
    let res = strg.list_ranges(date).await;

    match res {
        Ok(Some(rr)) => Ok(Json(rr)),
        Ok(None) => Err(ListError::NotFound),
        Err(err) => {
            error!(err = err.to_string(), "list time ranges");
            Err(ListError::Internal)
        }
    }
}

pub type ListResult<T> = Result<Json<Vec<T>>, ListError>;

pub enum ListError {
    NotFound,
    Internal,
}

impl IntoResponse for ListError {
    fn into_response(self) -> Response {
        let resp = match self {
            ListError::NotFound => StatusCode::NOT_FOUND,
            ListError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        resp.into_response()
    }
}

pub async fn create(
    State(strg): State<TimeRangeStorage>,
    Path(date): Path<NaiveDate>,
    Json(range): Json<TimeRange>,
) -> impl IntoResponse {
    let res = strg.put_range(date, range).await;

    match res {
        Ok(_) => StatusCode::CREATED,
        Err(err) => {
            error!(err = err.to_string(), "create time range");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
