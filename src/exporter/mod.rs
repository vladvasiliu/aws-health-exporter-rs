use crate::config::{Config, TLS};
use crate::exporter::error::Result;
use crate::scraper::Scraper;
use log::warn;
use prometheus::{gather, opts, Encoder, IntGauge, Registry, TextEncoder};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::result::Result as StdResult;
use std::sync::Arc;
use warp::Filter;

mod error;

pub struct Exporter {
    socket_address: SocketAddr,
    tls_config: Option<TLS>,
    scraper: Arc<Scraper>,
}

impl Exporter {
    pub fn new(config: Config) -> Result<Self> {
        let scraper = Arc::new(Scraper::new(&config)?);

        Ok(Self {
            socket_address: config.socket_addr,
            tls_config: config.tls_config,
            scraper,
        })
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

        let server = warp::serve(route);
        match &self.tls_config {
            Some(tls_config) => {
                let tls_server = server
                    .tls()
                    .key_path(&tls_config.key)
                    .cert_path(&tls_config.cert);
                tls_server.bind(self.socket_address).await;
            }
            None => server.try_bind(self.socket_address).await,
        }
    }
}

async fn scrape(scraper: Arc<Scraper>) -> StdResult<impl warp::Reply, Infallible> {
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
