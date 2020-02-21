use log::warn;
use prometheus::{gather, opts, Encoder, IntGauge, Registry, TextEncoder};
use std::convert::Infallible;
use warp::Filter;

use crate::scraper::Scraper;

pub struct Exporter {}

impl Exporter {
    pub async fn work() {
        let home = warp::path::end().map(|| warp::reply::html(HOME_PAGE));
        let status = warp::path("status").map(|| warp::reply::html(STATUS_PAGE));
        let metrics = warp::path("metrics").and_then(scrape);
        let route = home.or(status).or(metrics);

        warp::serve(route).run(([127, 0, 0, 1], 3030)).await;
    }
}

async fn scrape() -> Result<impl warp::Reply, Infallible> {
    let regions: Option<Vec<String>> = Some(vec![
        "eu-west-1".to_string(),
        "eu-central-1".to_string(),
        "eu-west-3".to_string(),
        "global".to_string(),
    ]);
    let scraper = Scraper::new(regions);
    let registry = Registry::new();

    let status_opts = opts!(
        "aws_health_events_success",
        "Whether retrieval of health events from AWS API was successful"
    );
    let status_gauge = IntGauge::with_opts(status_opts).unwrap();

    match scraper.describe_events().await {
        Ok(event_metrics) => {
            registry.register(Box::new(event_metrics)).unwrap();
            status_gauge.set(1);
        }
        Err(err) => warn!("{}", err),
    }
    registry.register(Box::new(status_gauge)).unwrap();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let mut metric_families = gather();
    metric_families.extend(registry.gather());
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}

static HOME_PAGE: &str = "
    <html>
        <head />
        <body>
            <ul>
                <li><a href=\"/status\">Exporter status</a></li>
                <li><a href=\"/metrics\">Metrics</a></li>
            </ul>
        </body>
    </html>
    ";

static STATUS_PAGE: &str = "<html><head /><body>Ok</body></html>";
