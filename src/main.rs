mod exporter;
mod scraper;

use fern::colors::{Color, ColoredLevelConfig};

use crate::scraper::Scraper;

#[tokio::main]
async fn main() {
    setup_logger().unwrap();
    let regions: Option<Vec<String>> = Some(vec![
        "eu-west-1".to_string(),
        "eu-central-1".to_string(),
        "eu-west-3".to_string(),
        "global".to_string(),
    ]);
    let scraper = Scraper::new(regions);
    scraper.describe_events().await;

    exporter::Exporter::work();
}

fn setup_logger() -> Result<(), fern::InitError> {
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
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        //        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
