use std::sync::Arc;

use crate::domain::range::{
    GetResult, InsertResult, ListResult, Range, Repository as RepositoryTrait, RepositoryError,
};
use crate::infra::redb::{impl_from_err, impl_value};

use chrono::{Datelike, NaiveDate};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use tokio::task::JoinError;

const RANGES_BY_ID: TableDefinition<[u8; 16], Range> = TableDefinition::new("ranges");
const RANGES_BY_DAY: TableDefinition<u32, Vec<Range>> = TableDefinition::new("ranges_by_day");

pub struct Repository {
    db: Arc<Database>,
}

impl Repository {
    pub fn new(db: Arc<Database>) -> Result<Self, RepositoryError> {
        let txn = db.begin_write()?;
        txn.open_table(RANGES_BY_ID)?;
        txn.open_table(RANGES_BY_DAY)?;
        txn.commit()?;
        let repo = Self { db };
        Ok(repo)
    }
}

#[async_trait::async_trait]
impl RepositoryTrait for Repository {
    async fn list_for_day(&self, day: NaiveDate) -> ListResult {
        let db = self.db.clone();
        tokio::task::spawn_blocking(move || {
            let txn = db.begin_read()?;
            let table = txn.open_table(RANGES_BY_DAY)?;
            let value = table.get(naivedate2u32(&day))?;
            Ok(value.map(|v| v.value()).unwrap_or_default())
        })
        .await?
    }

    async fn get_by_id(&self, id: &uuid::Uuid) -> GetResult {
        let db = self.db.clone();
        let id = *id;
        tokio::task::spawn_blocking(move || {
            let txn = db.begin_read()?;
            let table = txn.open_table(RANGES_BY_ID)?;
            let value = table.get(id.as_bytes())?;
            Ok(value.map(|v| v.value()))
        })
        .await?
    }

    async fn insert(&self, range: &Range) -> InsertResult {
        let db = self.db.clone();
        let range = *range;
        tokio::task::spawn_blocking(move || {
            let txn = db.begin_write()?;
            {
                let mut table = txn.open_table(RANGES_BY_DAY)?;
                let key = naivedate2u32(&range.day());
                let mut ranges = table.get(key)?.map(|v| v.value()).unwrap_or_default();

                let intersects = ranges.iter().any(|x| range.intersects(x));
                if intersects {
                    return Err(RepositoryError::Intersects);
                }

                ranges.push(range);
                table.insert(&key, &ranges)?;
            }

            {
                let mut table = txn.open_table(RANGES_BY_ID)?;
                table.insert(range.id().as_bytes(), range)?;
            }
            txn.commit()?;
            Ok(())
        })
        .await?
    }
}

fn naivedate2u32(date: &NaiveDate) -> u32 {
    date.year() as u32 * 10000 + date.month() * 100 + date.day()
}

impl_value!(Range);

impl_from_err!(redb::TransactionError, RepositoryError, Database);
impl_from_err!(redb::TableError, RepositoryError, Database);
impl_from_err!(redb::CommitError, RepositoryError, Database);
impl_from_err!(redb::StorageError, RepositoryError, Database);
impl_from_err!(JoinError, RepositoryError, Database);

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::domain::time::Time;

    use super::*;

    #[tokio::test]
    async fn smoke() {
        let repo = create_repo();
        let id = uuid::Uuid::now_v7();

        let res = repo.get_by_id(&id).await.expect("get not exists");
        assert!(res.is_none());

        let expected = Range::new(
            NaiveDate::from_ymd(2026, 7, 3),
            Time::from_hms(1, 2, 3).expect("start time"),
            Some(Time::from_hms(4, 5, 6).expect("end time")),
        );
        repo.insert(&expected).await.expect("insert");

        let res = repo.get_by_id(&expected.id()).await.expect("get exists");
        assert!(matches!(res, Some(actual) if actual == expected));
    }

    fn create_repo() -> Repository {
        let file = tempfile::NamedTempFile::new().unwrap();
        let db = redb::Database::create(file.path()).expect("create DB");

        Repository::new(Arc::new(db)).expect("create repository")
    }
}
