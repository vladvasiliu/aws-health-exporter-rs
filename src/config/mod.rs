use clap::{crate_authors, crate_description, crate_name, crate_version, App, AppSettings, Arg};
use regex::Regex;
use rusoto_core::Region;
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
    pub role: Option<String>,
    pub role_region: Option<String>,
    pub use_organization: bool,
    pub regions: Option<Vec<String>>,
    pub services: Option<Vec<String>>,
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
                Arg::with_name("organization")
                    .long("organization")
                    .help("Query events for organization")
                    .takes_value(false)
                    .required(false),
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

        let regions = matches.values_of_lossy("region").map(|mut regions| {
            regions.push("global".to_string());
            regions.sort_unstable();
            regions.dedup();
            regions
        });

        let services = matches.values_of_lossy("service").map(|mut services| {
            services.sort_unstable();
            services.dedup();
            services
        });

        let tls_config = match (matches.value_of("tls_key"), matches.value_of("tls_cert")) {
            (Some(key), Some(cert)) => Some(TLS {
                key: key.to_string(),
                cert: cert.to_string(),
            }),
            _ => None,
        };

        let use_organization = matches.is_present("organization");

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
            use_organization,
        }
    }
}

fn validate_ip(ip: String) -> Result<(), String> {
    // The map at the end is required because Clap expects validators to return Result<(), String>.
    // https://docs.rs/clap/2.33.1/clap/struct.Arg.html#method.validator
    ip.parse::<SocketAddr>()
        .map_err(|err| format!("{}", err))
        .map(|_| ())
}

fn validate_region(region: String) -> Result<(), String> {
    if region == "all" {
        return Ok(());
    }
    Region::from_str(&region)
        .and(Ok(()))
        .map_err(|err| format!("{}", err))
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
