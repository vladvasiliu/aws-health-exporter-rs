use clap::{crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg};
use rusoto_core::Region;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

static DEFAULT_IP: &str = "[::]:9679";

#[derive(Debug)]
pub struct Config {
    pub socket_addr: SocketAddr,
    pub log_level: log::LevelFilter,
    pub regions: Option<Vec<String>>,
    pub services: Option<Vec<String>>,
    pub role: Option<String>,
    pub role_region: Option<String>,
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
            .arg(
                Arg::with_name("region")
                    .short("r")
                    .long("region")
                    .takes_value(true)
                    .required(false)
                    .help("Region for which to retrieve events")
                    .multiple(true)
                    .validator(validate_region),
            )
            .arg(
                Arg::with_name("service")
                    .short("s")
                    .long("service")
                    .takes_value(true)
                    .required(false)
                    .help("Service for which to retrieve events")
                    .multiple(true),
            )
            .arg(
                Arg::with_name("role")
                    .long("role")
                    .takes_value(true)
                    .help("Assume IAM Role")
                    .required(false),
            )
            .arg(
                Arg::with_name("role_region")
                    .long("role-region")
                    .help("Endpoint to use for calls to STS")
                    .takes_value(true)
                    .required(false)
                    .requires("role")
                    .validator(validate_region),
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

        let regions = matches.values_of_lossy("region").and_then(|mut regions| {
            regions.push("global".to_string());
            regions.sort_unstable();
            regions.dedup();
            Some(regions)
        });

        let services = matches.values_of_lossy("service").and_then(|mut services| {
            services.sort_unstable();
            services.dedup();
            Some(services)
        });

        Self {
            /// Works because the argument is validated
            socket_addr: matches.value_of("listen_host").unwrap().parse().unwrap(),
            log_level,
            version: crate_version!().to_string(),
            name: crate_name!().to_string(),
            regions,
            services,
            role: matches.value_of("role").map(|s| s.to_string()),
            role_region: matches.value_of("role_region").map(|s| s.to_string()),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = format!("Starting {} v{}\n", self.name, self.version);
        result.push_str(&format!("Listening on: {}\n", self.socket_addr));
        result.push_str(&format!("Log level: {}\n", self.log_level));
        if let Some(role) = &self.role {
            result.push_str(&format!("Role: {}\n", role));
        }
        if let Some(role_region) = &self.role_region {
            result.push_str(&format!("Role STS Endpoint: {}", role_region));
        }

        result.push_str("Regions:");
        match &self.regions {
            Some(regions) => {
                result.push_str("\n");
                for region in regions {
                    result.push_str(&format!("\t* {}\n", region));
                }
            }
            None => result.push_str(" All\n"),
        }

        result.push_str("Services:");
        match &self.services {
            Some(services) => {
                result.push_str("\n");
                for service in services {
                    result.push_str(&format!("\t* {}\n", service));
                }
            }
            None => result.push_str(" All\n"),
        }
        write!(f, "{}", result)
    }
}

fn validate_ip(ip: String) -> Result<(), String> {
    ip.parse::<SocketAddr>()
        .or_else(|err| Err(format!("{}", err)))
        .and_then(|_| Ok(()))
}

fn validate_region(region: String) -> Result<(), String> {
    if region == "all" {
        return Ok(());
    }
    Region::from_str(&region)
        .and(Ok(()))
        .or_else(|err| Err(format!("{}", err)))
}
