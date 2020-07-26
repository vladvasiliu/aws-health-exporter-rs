mod config;
mod exporter;
mod scraper;

use crate::exporter::Exporter;
use log::{error, info};

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() {
    let config = config::Config::from_args();
    setup_logger(config.log_level).unwrap();
    info!("Starting AWS Health Exporter v{}", config.version);
    info!("Listening on {}", config.socket_addr);

    match Exporter::new(config) {
        Ok(exporter) => exporter.work().await,
        Err(err) => error!("Failed to create exporter: {}", err),
    }
}

fn setup_logger(level: log::LevelFilter) -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[ {} ][ {:5} ][ {:15} ] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        //        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

// fn should_color_logs() {}
