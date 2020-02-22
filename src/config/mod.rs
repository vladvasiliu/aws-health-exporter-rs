use clap::{crate_authors, crate_version, App, AppSettings, Arg};

static DEFAULT_PORT: u64 = 3030;

#[derive(Debug)]
pub struct Config {
    pub port: u16,
}

impl Config {
    pub fn from_args() -> Self {
        let port_default = format!("{}", DEFAULT_PORT);

        let matches = App::new("AWS Health Exporter")
            .author(crate_authors!())
            .version(crate_version!())
            .about("Export AWS Health events to Prometheus")
            .settings(&[AppSettings::ColoredHelp, AppSettings::ColorAuto])
            .arg(
                Arg::with_name("listen_port")
                    .short("p")
                    .long("port")
                    .value_name("PORT")
                    .help("HTTP listen port")
                    .takes_value(true)
                    .required(false)
                    .default_value(&port_default)
                    .validator(int_valid_port),
            )
            .get_matches();

        Self {
            port: matches.value_of("listen_port").unwrap().parse().unwrap(),
        }
    }
}

fn int_valid_port(num: String) -> Result<(), String> {
    // Try to parse as i32 so that in case of overflow we can show a pretty error message.
    match num.parse::<i32>().or_else(|err| Err(format!("{}", err)))? {
        x if x < 0 || x > 65535 => Err("Must be between 1 an 65535".into()),
        _ => Ok(()),
    }
}
