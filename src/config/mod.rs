use clap::{crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg};
use std::fmt;
use std::net::SocketAddr;

static DEFAULT_IP: &str = "[::]:3030";

#[derive(Debug)]
pub struct Config {
    pub socket_addr: SocketAddr,
    pub log_level: log::LevelFilter,
    version: String,
    name: String,
}

impl Config {
    pub fn from_args() -> Self {
        let matches = App::new("AWS Health Exporter")
            .author(crate_authors!())
            .version(crate_version!())
            .about(crate_description!())
            .settings(&[AppSettings::ColoredHelp, AppSettings::ColorAuto])
            .arg(
                Arg::with_name("listen_host")
                    .short("l")
                    .long("listen")
                    .value_name("HOST:PORT")
                    .help("HTTP listen address")
                    .takes_value(true)
                    .required(false)
                    .default_value(DEFAULT_IP)
                    .validator(validate_ip),
            )
            .arg(
                Arg::with_name("debug")
                    .short("d")
                    .long("debug")
                    .help("Enable debug logging. Twice for trace logging.")
                    .takes_value(false)
                    .multiple(true)
                    .required(false),
            )
            .arg(
                Arg::with_name("quiet")
                    .short("q")
                    .long("quiet")
                    .help("Quiet logging. Only logs warning and above.")
                    .takes_value(false)
                    .required(false)
                    .conflicts_with("debug"),
            )
            .get_matches();

        let log_level = if matches.occurrences_of("debug") >= 2 {
            log::LevelFilter::Trace
        } else if cfg!(debug_assertions) || matches.occurrences_of("debug") == 1 {
            log::LevelFilter::Debug
        } else if matches.is_present("quiet") {
            log::LevelFilter::Warn
        } else {
            log::LevelFilter::Info
        };

        Self {
            socket_addr: matches.value_of("listen_host").unwrap().parse().unwrap(),
            log_level,
            version: crate_version!().to_string(),
            name: crate_name!().to_string(),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = format!("Starting {} v{}\n", self.name, self.version);
        result.push_str(&format!("Listening on: {}\n", self.socket_addr));
        result.push_str(&format!("Log level: {}\n", self.log_level));
        write!(f, "{}", result)
    }
}

fn validate_ip(ip: String) -> Result<(), String> {
    ip.parse::<SocketAddr>()
        .or_else(|err| Err(format!("{}", err)))
        .and_then(|_| Ok(()))
}
