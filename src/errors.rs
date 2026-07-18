macro_rules! define {
    ($name:ident, $msg:expr) => {
        #[derive(Debug, PartialEq)]
        pub struct $name;

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, $msg)
            }
        }

        impl std::error::Error for $name {}
    };
}

pub(crate) use define;
