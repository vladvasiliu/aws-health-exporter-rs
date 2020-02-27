use clap::{crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg};
use regex::Regex;
use rusoto_core::Region;
use std::fmt;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;

static DEFAULT_IP: &str = "[::]:9679";

#[derive(Debug)]
pub struct TLS {
    pub key: String,
    pub cert: String,
}

#[derive(Debug)]
pub struct Config {
    pub socket_addr: SocketAddr,
    pub log_level: log::LevelFilter,
    pub regions: Option<Vec<String>>,
    pub services: Option<Vec<String>>,
    pub role: Option<String>,
    pub role_region: Option<String>,
    pub tls_config: Option<TLS>,
    pub version: String,
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
                    .required(false)
                    .validator(validate_role_arn),
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
            .arg(
                Arg::with_name("tls_key")
                    .long("tls-key")
                    .help("Path to TLS certificate key")
                    .takes_value(true)
                    .required(false)
                    .requires("tls_cert")
                    .validator(validate_file_path),
            )
            .arg(
                Arg::with_name("tls_cert")
                    .long("tls-cert")
                    .help("Path to TLS certificate")
                    .takes_value(true)
                    .required(false)
                    .requires("tls_key")
                    .validator(validate_file_path),
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

        let tls_config = match (matches.value_of("tls_key"), matches.value_of("tls_cert")) {
            (Some(key), Some(cert)) => Some(TLS {
                key: key.to_string(),
                cert: cert.to_string(),
            }),
            _ => None,
        };

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
            tls_config,
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = format!("Starting {} v{}\n\n", self.name, self.version);
        result.push_str(&format!("{:<18}{}\n", "Listening on:", self.socket_addr));
        result.push_str(&format!("{:<18}{}\n", "Log level:", self.log_level));
        if let Some(role) = &self.role {
            result.push_str(&format!("{:<18}{}\n", "Role:", role));
        }
        if let Some(role_region) = &self.role_region {
            result.push_str(&format!("{:<18} {}", "Role STS Endpoint:", role_region));
        }

        result.push_str(&format!("{:<18}", "TLS config:"));
        if let Some(tls_config) = &self.tls_config {
            result.push_str("\n");
            result.push_str(&format!("  Key file:         {}\n", tls_config.key));
            result.push_str(&format!("  Certificate file: {}\n", tls_config.cert));
        } else {
            result.push_str("Off\n");
        }

        result.push_str(&format!("{:<18}", "Regions:"));
        match &self.regions {
            Some(regions) => {
                result.push_str("\n");
                for region in regions {
                    result.push_str(&format!("  * {}\n", region));
                }
            }
            None => result.push_str("All\n"),
        }

        result.push_str(&format!("{:<18}", "Services:"));
        match &self.services {
            Some(services) => {
                result.push_str("\n");
                for service in services {
                    result.push_str(&format!("  * {}\n", service));
                }
            }
            None => result.push_str("All\n"),
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

/// Validates that a given AWS Role ARN looks OK
///
/// The 20 character limit comes from an error raised when a short string is given.
/// The regular expression comes from the official docs:
/// https://docs.aws.amazon.com/IAM/latest/UserGuide/list_identityandaccessmanagement.html#identityandaccessmanagement-resources-for-iam-policies
fn validate_role_arn(role_arn: String) -> Result<(), String> {
    if role_arn.len() < 20 {
        return Err("Must have length greater than or equal to 20".to_string());
    }
    let arn_regex = Regex::new(r"(?i:arn:aws:iam::\d{12}:role/.*)").unwrap();
    if arn_regex.is_match(&role_arn) {
        Ok(())
    } else {
        Err("must be of the form `arn:aws:iam::123456789012:role/something`".to_string())
    }
}

fn validate_file_path(file_path: String) -> Result<(), String> {
    if Path::new(&file_path).is_file() {
        Ok(())
    } else {
        Err(format!("{} is not a file", file_path))
    }
}
