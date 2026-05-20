use std::net::{IpAddr, Ipv4Addr};
use std::ops::RangeInclusive;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// An address to bind.
    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    pub address: IpAddr,
    /// Port to bind.
    #[arg(short, long, default_value_t = 3000, value_parser = port_in_range)]
    pub port: u16,
    /// Path to store database.
    #[arg(short, long, default_value = "/var/wtimer/db.redb")]
    pub db: PathBuf,
    /// Logging level.
    #[arg(short, long, default_value_t = tracing::Level::INFO)]
    pub log_level: tracing::Level,
}

pub fn load() -> Config {
    Config::parse()
}

fn port_in_range(s: &str) -> Result<u16, String> {
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

const PORT_RANGE: RangeInclusive<usize> = 1..=65535;
