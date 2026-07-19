pub mod range;

use std::path::Path;

pub fn connect<T: AsRef<Path>>(path: T) -> Result<redb::Database, redb::DatabaseError> {
    redb::Database::create(path)
}

macro_rules! impl_value {
    ($type:ident) => {
        impl redb::Value for $type {
            type SelfType<'a> = Self;
            type AsBytes<'a> = Vec<u8>;

            fn fixed_width() -> Option<usize> {
                None
            }

            fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
            where
                Self: 'a,
            {
                postcard::from_bytes::<$type>(data).expect("desirializaton failed")
            }

            fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
            where
                Self: 'b,
            {
                postcard::to_stdvec(value).expect("serialization failed")
            }

            fn type_name() -> redb::TypeName {
                redb::TypeName::new("$type")
            }
        }
    };
}

macro_rules! impl_from_err {
    ($redb_err:ty, $domain_err:ident, $name:ident) => {
        impl From<$redb_err> for $domain_err {
            fn from(err: $redb_err) -> Self {
                $domain_err::$name(err.to_string())
            }
        }
    };
}

use impl_from_err;
use impl_value;
