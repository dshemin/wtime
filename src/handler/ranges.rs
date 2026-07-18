use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{
    domain::range::{Range, Repository, RepositoryError},
    handler::Result,
};
use crate::{
    domain::time::{Time, TimeError},
    handler::ErrorResponse,
};

pub async fn list(
    State(repo): State<Arc<dyn Repository>>,
    Path(date): Path<NaiveDate>,
) -> Result<Vec<Range>> {
    let res = repo.list_for_day(date).await?;
    Ok(Json(res))
}

pub async fn create(
    State(repo): State<Arc<dyn Repository>>,
    Json(req): Json<CreateRangeRequest>,
) -> Result<CreateRangeResponse> {
    let start = Time::from_hms(req.start.hour, req.start.minute, req.start.seconds)?;
    let end = req
        .end
        .map(|x| Time::from_hms(x.hour, x.minute, x.seconds))
        .transpose()?;
    let range = Range::new(req.date, start, end);

    repo.insert(&range).await?;
    let resp = CreateRangeResponse { id: range.id() };
    Ok(Json(resp))
}

#[derive(Deserialize)]
pub struct CreateRangeRequest {
    date: NaiveDate,
    start: CreateRangeTime,
    end: Option<CreateRangeTime>,
}

#[derive(Deserialize)]
pub struct CreateRangeTime {
    hour: u32,
    minute: u32,
    seconds: u32,
}

#[derive(Serialize)]
pub struct CreateRangeResponse {
    id: uuid::Uuid,
}

impl From<RepositoryError> for ErrorResponse {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::Database(err) => ErrorResponse::internal(err.to_string()),
            RepositoryError::Serialization(err) => ErrorResponse::internal(err.to_string()),
            RepositoryError::Intersects => {
                ErrorResponse::bad_request("range intersects with exists")
            }
        }
    }
}

impl From<TimeError> for ErrorResponse {
    fn from(value: TimeError) -> Self {
        // I might use simple `ErrorResponse::bad_request(value.to_string())` but
        // it a fragile.
        match value {
            TimeError::Hours | TimeError::Minutes | TimeError::Seconds => {
                ErrorResponse::bad_request(value.to_string())
            }
        }
    }
}
