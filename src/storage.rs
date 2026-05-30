use std::path::Path;
use std::sync::Arc;

use chrono::{Datelike, NaiveDate};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition, TypeName, Value};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct TimeRangeStorage {
    db: Arc<Database>,
}

impl TimeRangeStorage {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<TimeRangeStorage> {
        let db = Database::create(path)?;
        let db = Arc::new(db);
        let strg = TimeRangeStorage { db };
        Ok(strg)
    }

    pub async fn put_range(&self, at: NaiveDate, range: TimeRange) -> anyhow::Result<()> {
        let db = self.db.clone();
        let key = Self::date_to_key(&at);
        let handle = tokio::task::spawn_blocking(move || {
            let txn = db.begin_write()?;
            {
                let mut table = txn.open_table(TABLE_RANGES)?;
                let mut ranges = match table.get(key)? {
                    Some(g) => g.value().to_vec(),
                    None => Vec::with_capacity(1),
                };
                ranges.push(range);
                table.insert(key, ranges)?;
            }
            txn.commit()?;
            Ok(())
        });
        handle.await?
    }

    pub async fn list_ranges(&self, at: NaiveDate) -> anyhow::Result<Option<Vec<TimeRange>>> {
        let db = self.db.clone();
        let key = Self::date_to_key(&at);
        let handle = tokio::task::spawn_blocking(move || {
            let txn = db.begin_read()?;
            let table = txn.open_table(TABLE_RANGES)?;
            let res = table.get(key)?.map(|g| g.value().to_vec());
            Ok(res)
        });
        handle.await?
    }

    fn date_to_key(at: &NaiveDate) -> u32 {
        let year = at.year() as u32;
        let month = at.month0();
        let day = at.day0();

        year << 16 | month << 8 | day
    }
}

const TABLE_RANGES: TableDefinition<u32, Vec<TimeRange>> = TableDefinition::new("ranges");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    start: Time,
    end: Option<Time>,
}

impl Value for TimeRange {
    type SelfType<'a>
        = TimeRange
    where
        Self: 'a;

    type AsBytes<'a>
        = [u8; 4]
    where
        Self: 'a;

    fn fixed_width() -> Option<usize> {
        Some(4) // The number of u8 items not a size of the type!
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        if data.len() == 0 {
            return TimeRange {
                start: Time::from_bytes(0, 0),
                end: None,
            };
        }
        let start = Time::from_bytes(data[0], data[1]);
        let end = match data[2] {
            128 => None,
            v => Some(Time::from_bytes(v, data[3])),
        };
        TimeRange { start, end }
    }

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'b,
    {
        match &value.end {
            None => [value.start.hours(), value.start.minutes(), 128, 0],
            Some(end) => [
                value.start.hours(),
                value.start.minutes(),
                end.hours(),
                end.minutes(),
            ],
        }
    }

    fn type_name() -> TypeName {
        TypeName::new("TimeRange")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Time {
    hour: u8,
    minute: u8,
}

pub type TimeResult = Result<Time, TimeError>;

impl Time {
    pub fn new(hour: u8, minute: u8) -> TimeResult {
        Self::validate_hour(hour)?;
        Self::validate_minute(minute)?;

        let t = Self { hour, minute };
        Ok(t)
    }

    pub fn hours(&self) -> u8 {
        self.hour
    }
    pub fn minutes(&self) -> u8 {
        self.minute
    }

    fn from_bytes(h: u8, m: u8) -> Time {
        Self::new(h, m).unwrap()
    }

    fn validate_hour(h: u8) -> Result<(), TimeError> {
        Self::validate(h, 24, TimeError::Hour)
    }

    fn validate_minute(m: u8) -> Result<(), TimeError> {
        Self::validate(m, 60, TimeError::Minute)
    }

    fn validate<F>(v: u8, max: u8, err_fn: F) -> Result<(), TimeError>
    where
        F: FnOnce(u8) -> TimeError,
    {
        if v <= max {
            Ok(())
        } else {
            let e = err_fn(v);
            Err(e)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TimeError {
    #[error("invalid hour {0}, valid [0, 24]")]
    Hour(u8),
    #[error("ivalid minut {0}, valid [0, 60]")]
    Minute(u8),
}
