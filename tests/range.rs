mod fixture;

use std::fmt::format;

use chrono::NaiveDate;
use serde_json::json;
use wtimer_lib::domain::range::Range;

use fixture::Fixture;

#[tokio::test]
async fn list_ranges_empty() {
    let fixture = Fixture::new().await;

    let res = fixture.get("/api/v1/ranges?date=2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert!(ranges.is_empty());
}

#[tokio::test]
async fn create_range() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    assert!(body["id"].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn create_and_list_ranges() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let res = fixture.get("/api/v1/ranges?date=2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert_eq!(ranges.len(), 1);
    assert_eq!(
        ranges[0].day(),
        NaiveDate::from_ymd_opt(2026, 7, 15).unwrap()
    );
}

#[tokio::test]
async fn create_intersecting_range_fails() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 12, "minute": 0, "seconds": 0 },
            "end": { "hour": 14, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_bad_request();
}

#[tokio::test]
async fn create_range_without_end() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let body: serde_json::Value = res.json();
    assert!(body["id"].as_str().unwrap().len() > 0);

    let res = fixture.get("/api/v1/ranges?date=2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert_eq!(ranges.len(), 1);
    assert!(ranges[0].end().is_none());
}

#[tokio::test]
async fn create_range_invalid_time_fails() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 25, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_bad_request();
}

#[tokio::test]
async fn update_range() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = fixture
        .put(format!("/api/v1/ranges/{id}"))
        .json(&json!({
            "id": id,
            "day": "2026-07-15",
            "start": { "hour": 10, "minute": 30, "seconds": 0 },
            "end": { "hour": 18, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let res = fixture.get("/api/v1/ranges?date=2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].start().seconds(), 10 * 3600 + 30 * 60);
    assert_eq!(ranges[0].end().unwrap().seconds(), 18 * 3600);
}

#[tokio::test]
async fn update_nonexistent_range_fails() {
    let fixture = Fixture::new().await;

    let res = fixture
        .put("/api/v1/ranges/00000000-0000-0000-0000-000000000000")
        .json(&json!({
            "day": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_not_found();
}

#[tokio::test]
async fn update_intersecting_range_fails() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 12, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 13, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = fixture
        .put(format!("/api/v1/ranges/{id}"))
        .json(&json!({
            "day": "2026-07-15",
            "start": { "hour": 10, "minute": 0, "seconds": 0 },
            "end": { "hour": 14, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_bad_request();
}

#[tokio::test]
async fn update_range_remove_end() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = fixture
        .put(format!("/api/v1/ranges/{id}"))
        .json(&json!({
            "day": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let res = fixture.get("/api/v1/ranges?date=2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert_eq!(ranges.len(), 1);
    assert!(ranges[0].end().is_none());
}

#[tokio::test]
async fn update_range_invalid_time_fails() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = fixture
        .put(format!("/api/v1/ranges/{id}"))
        .json(&json!({
            "day": "2026-07-15",
            "start": { "hour": 25, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_bad_request();
}

#[tokio::test]
async fn delete_range() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();
    let id = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = fixture.delete(&format!("/api/v1/ranges/{}", id)).await;
    res.assert_status_ok();

    let res = fixture.get("/api/v1/ranges?date=2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert!(ranges.is_empty());
}

#[tokio::test]
async fn delete_nonexistent_range_succeeds() {
    let fixture = Fixture::new().await;

    let res = fixture
        .delete("/api/v1/ranges/00000000-0000-0000-0000-000000000000")
        .await;
    res.assert_status_ok();
}

#[tokio::test]
async fn delete_range_and_keep_others() {
    let fixture = Fixture::new().await;

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 9, "minute": 0, "seconds": 0 },
            "end": { "hour": 12, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();
    let id1 = res.json::<serde_json::Value>()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let res = fixture
        .post("/api/v1/ranges")
        .json(&json!({
            "date": "2026-07-15",
            "start": { "hour": 13, "minute": 0, "seconds": 0 },
            "end": { "hour": 17, "minute": 0, "seconds": 0 },
        }))
        .await;
    res.assert_status_ok();

    let res = fixture.delete(&format!("/api/v1/ranges/{}", id1)).await;
    res.assert_status_ok();

    let res = fixture.get("/api/v1/ranges?date=2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].start().seconds(), 13 * 3600);
}
