use clap::{crate_authors, crate_version, App, AppSettings, Arg};
use std::net::SocketAddr;

static DEFAULT_IP: &str = "[::]:3030";

#[derive(Debug)]
pub struct Config {
    pub socket_addr: SocketAddr,
}

impl Config {
    pub fn from_args() -> Self {
        let matches = App::new("AWS Health Exporter")
            .author(crate_authors!())
            .version(crate_version!())
            .about("Export AWS Health events to Prometheus")
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
            .get_matches();

        Self {
            socket_addr: matches.value_of("listen_host").unwrap().parse().unwrap(),
        }
    }
}

fn validate_ip(ip: String) -> Result<(), String> {
    ip.parse::<SocketAddr>()
        .or_else(|err| Err(format!("{}", err)))
        .and_then(|_| Ok(()))
}
