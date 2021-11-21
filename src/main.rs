#[macro_use]
extern crate lazy_static;

use tracing::{error, info};

use crate::exporter::Exporter;

mod config;
mod exporter;
mod scraper;

#[tokio::main]
async fn main() {
    let config = config::Config::from_args();
    tracing_subscriber::fmt::init();
    info!(
        "AWS Health Exporter v{} - Listening on {}.",
        config.version, config.socket_addr
    );

    match Exporter::new(config) {
        Ok(exporter) => exporter.work().await,
        Err(err) => error!("Failed to create exporter: {}", err),
    }
}
