mod scraper;

use crate::scraper::Scraper;

#[tokio::main]
async fn main() {
    let scraper = Scraper::new(None);
    scraper.describe_events().await;
}
