use log::{info, warn};
use prometheus::{gather, opts, Encoder, IntGauge, Registry, TextEncoder};
use std::convert::Infallible;
use std::net::SocketAddr;
use warp::Filter;

use crate::config::Config;
use crate::scraper::Scraper;
use std::sync::Arc;

pub struct Exporter {
    socket_address: SocketAddr,
    scraper: Arc<Scraper>,
}

impl Exporter {
    pub fn new(config: Config) -> Self {
        let scraper = Arc::new(Scraper::new(config.regions));

        Self {
            socket_address: config.socket_addr,
            scraper,
        }
    }

    pub async fn work(&self) {
        let scraper = self.scraper.clone();
        let home = warp::path::end().map(|| warp::reply::html(HOME_PAGE));
        let status = warp::path("status").map(|| warp::reply::html(STATUS_PAGE));
        let metrics = warp::path("metrics").and_then(move || {
            let scraper = scraper.clone();
            scrape(scraper)
        });
        let route = home.or(status).or(metrics);

        let srv = warp::serve(route).try_bind(self.socket_address);
        info!("Listening on {}", self.socket_address);
        srv.await;
    }
}

async fn scrape(scraper: Arc<Scraper>) -> Result<impl warp::Reply, Infallible> {
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
