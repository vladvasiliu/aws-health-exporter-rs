mod config;
mod exporter;
mod scraper;

use crate::exporter::Exporter;
use fern::colors::{Color, ColoredLevelConfig};

#[tokio::main]
async fn main() {
    setup_logger().unwrap();
    let config = config::Config::from_args();

    let exporter = Exporter::new(config);
    exporter.work().await;
}

fn setup_logger() -> Result<(), fern::InitError> {
    let default_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    let colors = ColoredLevelConfig::new()
        .debug(Color::Cyan)
        .info(Color::Blue)
        .warn(Color::Yellow)
        .error(Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[ {} ][ {:5} ][ {:15} ] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(default_level)
        .chain(std::io::stdout())
        //        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
