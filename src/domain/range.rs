use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::time::Time;

/// A time range.
///
/// Each range leave inside the single day and might be without end day.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    id: Uuid,
    day: NaiveDate,
    start: Time,
    end: Option<Time>,
}

impl Range {
    /// Create a new range
    ///
    /// # Example
    ///
    /// ```
    /// # use wtimer_lib::domain::range::Range;
    /// # use wtimer_lib::domain::time::Time;
    /// # use chrono::NaiveDate;
    ///
    /// let range = Range::new(
    ///     NaiveDate::from_ymd(2023, 6, 25),
    ///     Time::from_hms(1, 2, 3).unwrap(),
    ///     Some(Time::from_hms(4, 5, 6).unwrap()),
    /// );
    /// assert!(!range.id().is_nil());
    /// assert_eq!(range.day(), NaiveDate::from_ymd(2023, 6, 25));
    /// assert_eq!(range.start(), Time::from_hms(1, 2, 3).unwrap());
    /// assert_eq!(range.end(), Some(Time::from_hms(4, 5, 6).unwrap()));
    /// ```
    pub fn new(day: NaiveDate, start: Time, end: Option<Time>) -> Self {
        let id = Uuid::now_v7();
        Self::new_with_id(id, day, start, end)
    }

    pub fn new_with_id(id: Uuid, day: NaiveDate, start: Time, end: Option<Time>) -> Self {
        Self {
            id,
            day,
            start,
            end,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn day(&self) -> NaiveDate {
        self.day
    }

    pub fn start(&self) -> Time {
        self.start
    }

    pub fn end(&self) -> Option<Time> {
        self.end
    }

    pub fn intersects(&self, other: &Self) -> bool {
        // Handle next cases:
        // * x1---------y1
        //       x2---y2
        //
        // *     x1---y1
        //   x2---------y2
        //
        // *    x1------y1
        //   x2------y2
        //
        // * x1------y1
        //      x2------y2
        //
        // * x1---------y1
        //   x2---------y2
        //
        // * x1---------y1
        //   x2-----y2
        //
        // * x1---------y1
        //      x2------y2
        //
        // And don't forget for not closed ranges.
        if self.day != other.day {
            return false;
        }
        let cur_start = self.start;
        let cur_end = self.end.unwrap_or(Time::MAX);

        let other_start = other.start;
        let other_end = other.end.unwrap_or(Time::MAX);

        cur_start < other_end && other_start < cur_end
    }

    pub fn intersects_iter<'a, T>(&'a self, ranges: T) -> bool
    where
        T: IntoIterator<Item = &'a Range>,
    {
        // We should keep in mind that self might be in the given array of ranges.
        ranges
            .into_iter()
            .any(|x| x.id() != self.id && self.intersects(x))
    }
}

#[async_trait::async_trait]
pub trait Repository: Send + Sync {
    async fn list_for_day(&self, day: NaiveDate) -> ListResult;
    async fn get_by_id(&self, id: &uuid::Uuid) -> GetResult;
    async fn insert(&self, range: &Range) -> InsertResult;
    async fn update(&self, range: &Range) -> UpdateResult;
    async fn delete(&self, id: &uuid::Uuid) -> DeleteResult;
}

pub type ListResult = Result<Vec<Range>, RepositoryError>;
pub type GetResult = Result<Option<Range>, RepositoryError>;
pub type InsertResult = Result<(), RepositoryError>;
pub type UpdateResult = Result<(), RepositoryError>;
pub type DeleteResult = Result<(), RepositoryError>;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("range not exists")]
    NotExists,

    #[error("intersects with another range within same day")]
    Intersects,
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! range {
        ($day:expr, $start:expr, $end:expr) => {{
            let start = Time::new($start).unwrap();
            let end = Time::new($end).unwrap();
            Range::new($day, start, Some(end))
        }};
        ($day:expr, $start:expr) => {{
            let start = Time::new($start).unwrap();
            Range::new($day, start, None)
        }};
    }

    mod intersects {
        use super::*;

        static DAY1: NaiveDate = NaiveDate::from_ymd(2023, 6, 25);
        static DAY2: NaiveDate = NaiveDate::from_ymd(2026, 6, 25);

        macro_rules! case {
            ($name:ident, ($r1:expr, $r2:expr)) => {
                #[test]
                fn $name() {
                    assert!($r1.intersects(&$r2));
                    assert!($r2.intersects(&$r1));
                }
            };
            ($name:ident, !($r1:expr, $r2:expr)) => {
                #[test]
                fn $name() {
                    assert!(!$r1.intersects(&$r2));
                    assert!(!$r2.intersects(&$r1));
                }
            };
        }

        case!(same, (range!(DAY1, 5, 10), range!(DAY1, 5, 10)));
        case!(one_starts_early, (range!(DAY1, 1, 10), range!(DAY1, 5, 10)));
        case!(one_end_later, (range!(DAY1, 5, 15), range!(DAY1, 5, 10)));
        case!(
            one_inside_another,
            (range!(DAY1, 7, 9), range!(DAY1, 5, 10))
        );
        case!(
            one_inside_another_same_start,
            (range!(DAY1, 5, 9), range!(DAY1, 5, 10))
        );
        case!(
            one_inside_another_same_end,
            (range!(DAY1, 7, 10), range!(DAY1, 5, 10))
        );

        // Same cases as above but with different days.
        case!(
            same_but_different_day,
            !(range!(DAY1, 5, 10), range!(DAY2, 5, 10))
        );
        case!(
            one_starts_early_but_different_day,
            !(range!(DAY1, 1, 10), range!(DAY2, 5, 10))
        );
        case!(
            one_end_later_but_different_day,
            !(range!(DAY1, 5, 15), range!(DAY2, 5, 10))
        );
        case!(
            one_inside_another_but_different_day,
            !(range!(DAY1, 7, 9), range!(DAY2, 5, 10))
        );
        case!(
            one_inside_another_same_start_but_different_day,
            !(range!(DAY1, 5, 9), range!(DAY2, 5, 10))
        );
        case!(
            one_inside_another_same_end_but_different_day,
            !(range!(DAY1, 7, 10), range!(DAY2, 5, 10))
        );

        case!(not_same, !(range!(DAY1, 1, 5), range!(DAY1, 10, 15)));
        case!(
            one_starts_after_another,
            !(range!(DAY1, 5, 10), range!(DAY1, 10, 15))
        );

        // Make sure intersections works correctly for ranges without end time.
        case!(same_without_end, (range!(DAY1, 1), range!(DAY1, 1)));
        case!(
            one_inside_another_without_end,
            (range!(DAY1, 5, 10), range!(DAY1, 1))
        );

        // The same as above but with different day.
        case!(
            same_without_end_different_day,
            !(range!(DAY1, 1), range!(DAY2, 1))
        );
        case!(
            one_inside_another_without_end_different_day,
            !(range!(DAY1, 5, 10), range!(DAY2, 1))
        );
    }
}
