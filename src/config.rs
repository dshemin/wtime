use std::net::{IpAddr, Ipv4Addr};
use std::ops::RangeInclusive;
use std::path::{PathBuf, absolute};

use clap::Parser;

#[derive(Parser, Debug, PartialEq)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// An address to bind.
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    pub address: IpAddr,
    /// Port to bind.
    #[arg(short, long, default_value_t = 3000, value_parser = port_parser)]
    pub port: u16,
    /// Path to store database.
    #[arg(short, long, default_value = "/tmp/db.redb", value_parser = db_path_parser)]
    pub db: PathBuf,
    /// Logging level.
    #[arg(short, long, default_value_t = tracing::Level::INFO)]
    pub log_level: tracing::Level,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 3000,
            db: PathBuf::from("/tmp/db.redb"),
            log_level: tracing::Level::INFO,
        }
    }
}

pub fn load() -> Config {
    Config::parse()
}

fn port_parser(s: &str) -> Result<u16, String> {
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a port number"))?;
    if PORT_RANGE.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "port not in range {}-{}",
            PORT_RANGE.start(),
            PORT_RANGE.end()
        ))
    }
}

fn db_path_parser(s: &str) -> Result<PathBuf, String> {
    let path: PathBuf = s.parse().map_err(|_| format!("`{s}` is not a path"))?;
    let path =
        absolute(path).map_err(|err| format!("failed to make path `{s}` absolute: `{err}`"))?;

    if let Some(dir) = path.parent()
        && dir.exists()
        && !dir.is_dir()
    {
        return Err(format!("`{s}` base directory is not a directory"));
    }

    if path.exists() && path.is_dir() {
        return Err(format!("`{s}` is a directory"));
    }

    Ok(path)
}

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::{NamedTempFile, tempdir};
    use tracing::Level;

    macro_rules! parse {
        () => {{
            let args = vec!["test_bin"];
            Config::try_parse_from(args)
        }};
        ($($arg:expr),*) => {{
            let args = vec!["test_bin", $($arg),*];
            Config::try_parse_from(args)
        }};
    }

    macro_rules! parse_success {
        () => {{
            parse!().expect("parse")
        }};
        ($($arg:expr),*) => {{
            parse!($($arg),*).expect("parse")
        }};
    }

    macro_rules! parse_fail {
        () => {
            let actual = parse!().err().expect("no error").kind();
            assert_eq!(clap::error::ErrorKind::ValueValidation, actual);
        };
        ($($arg:expr),*) => {{
            let actual = parse!($($arg),*).err().expect("no error").kind();
            assert_eq!(clap::error::ErrorKind::ValueValidation, actual);
        }};
    }

    #[test]
    fn config_empty() {
        let expected = Config::default();

        let actual = parse_success!();

        assert_eq!(expected, actual);
    }

    #[test]
    fn config_set_all() {
        let expected = Config {
            address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 1234,
            db: PathBuf::from("/tmp/foo/bar/db.redb"),
            log_level: tracing::Level::TRACE,
        };

        let actual = parse_success!(
            "--address",
            "127.0.0.1",
            "--port",
            "1234",
            "--db",
            "/tmp/foo/bar/db.redb",
            "--log-level",
            "trace"
        );

        assert_eq!(expected, actual);
    }

    macro_rules! config_test_fail {
        ($name:ident, $($arg:expr),*) => {
            #[test]
            fn $name() {
                parse_fail!($($arg),*);
            }
        };
    }

    config_test_fail!(address_invalid, "--address", "invalid");
    config_test_fail!(port_invalid_less_min, "--port", "0");
    config_test_fail!(port_invalid_greater_min, "--port", "65537");

    #[test]
    fn db_path_invalid_parent_not_a_dir() {
        let file = NamedTempFile::new().expect("create tempfile");
        let db_path = file.path().join("dir");

        parse_fail!("--db", db_path.to_str().unwrap());
    }

    #[test]
    fn db_path_invalid_is_a_dir() {
        let dir = tempdir().expect("create temporary dir");
        let db_path = dir.path();

        parse_fail!("--db", db_path.to_str().unwrap());
    }

    macro_rules! log_level_test {
        ($name:ident, $level:expr) => {
            #[test]
            fn $name() {
                let mut expected = Config::default();
                expected.log_level = $level;
                let actual = parse_success!("--log-level", $level.as_str());
                assert_eq!(expected, actual);
            }
        };
    }

    log_level_test!(log_level_trace, Level::TRACE);
    log_level_test!(log_level_debug, Level::DEBUG);
    log_level_test!(log_level_info, Level::INFO);
    log_level_test!(log_level_warn, Level::WARN);
    log_level_test!(log_level_error, Level::ERROR);

    #[test]
    fn log_level_invalid() {
        parse_fail!("--log-level", "foobar");
    }
}
