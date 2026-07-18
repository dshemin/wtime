use std::sync::Arc;

use axum::extract::Path;
use axum_test::{TestRequest, TestServer};
use redb::Database;
use tempfile::tempdir;

use wtimer_lib::*;

pub struct Fixture {
    db: Arc<Database>,
    srv: TestServer,
}

macro_rules! from_test_server {
    ($name:ident) => {
        pub fn $name(&self, path: &str) -> TestRequest {
            self.srv.$name(path)
        }
    };
}

impl Fixture {
    pub async fn new() -> Self {
        let dir = tempdir().unwrap();
        let db_file = dir.path().join("test.redb");
        let db = Arc::new(infra::redb::connect(db_file).unwrap());
        let range_repo = infra::redb::range::Repository::new(db.clone()).unwrap();
        let app = server::setup(range_repo);
        let srv = TestServer::new(app);

        Self { db, srv }
    }

    from_test_server!(get);
    from_test_server!(post);
}
