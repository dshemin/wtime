mod fixture;

use chrono::NaiveDate;
use serde_json::json;
use wtimer_lib::domain::range::Range;

use fixture::Fixture;

#[tokio::test]
async fn list_ranges_empty() {
    let fixture = Fixture::new().await;

    let res = fixture.get("/api/v1/ranges/2026-07-15").await;
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

    let res = fixture.get("/api/v1/ranges/2026-07-15").await;
    res.assert_status_ok();

    let ranges = res.json::<Vec<Range>>();
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0].day(), NaiveDate::from_ymd(2026, 7, 15));
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

    let res = fixture.get("/api/v1/ranges/2026-07-15").await;
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
