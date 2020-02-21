mod exporter;
mod scraper;

use crate::scraper::Scraper;

#[tokio::main]
async fn main() {
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
