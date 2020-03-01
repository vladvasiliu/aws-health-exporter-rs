use crate::config::{Config, TLS};
use crate::exporter::error::Result;
use crate::scraper::Scraper;
use clap::crate_version;
use log::warn;
use prometheus::{
    gather, labels, opts, register, Encoder, IntCounterVec, IntGauge, Registry, TextEncoder,
};
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
    exporter_metrics: Arc<IntCounterVec>,
}

impl Exporter {
    pub fn new(config: Config) -> Result<Self> {
        let scraper = Arc::new(Scraper::new(&config)?);
        let exporter_metrics = Arc::new(create_exporter_metrics()?);
        create_info_metric(&config)?;

        Ok(Self {
            socket_address: config.socket_addr,
            tls_config: config.tls_config,
            scraper,
            exporter_metrics,
        })
    }

    pub async fn work(&self) {
        let scraper = self.scraper.clone();
        let metrics_family = self.exporter_metrics.clone();
        let home = warp::path::end().map(|| warp::reply::html(HOME_PAGE.as_str()));
        let status = warp::path("status").map(|| warp::reply::html(STATUS_PAGE));
        let metrics = warp::path("metrics").and_then(move || {
            let scraper = scraper.clone();
            let metrics_family = metrics_family.clone();
            scrape(scraper, metrics_family)
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

fn create_info_metric(config: &Config) -> Result<()> {
    let metric_opts = opts!(
        "aws_health_exporter_info",
        "Exporter information",
        labels! {"version" => &config.version,}
    );
    let metric = IntGauge::with_opts(metric_opts).unwrap();
    metric.set(1);
    register(Box::new(metric))?;
    Ok(())
}

fn create_exporter_metrics() -> Result<IntCounterVec> {
    let exporter_opts = opts!(
        "http_requests",
        "Number of HTTP requests received by the exporter"
    );
    let labels = ["status"];
    let exporter_metrics = IntCounterVec::new(exporter_opts, &labels)?;
    register(Box::new(exporter_metrics.clone()))?;
    Ok(exporter_metrics)
}

async fn scrape(
    scraper: Arc<Scraper>,
    exporter_metrics_family: Arc<IntCounterVec>,
) -> StdResult<impl warp::Reply, Infallible> {
    let registry = Registry::new();
    let status_opts = opts!(
        "aws_health_events_success",
        "Whether retrieval of health events from AWS API was successful"
    );
    let status_gauge = IntGauge::with_opts(status_opts).unwrap();

    let labels: &[&str];
    match scraper.describe_events().await {
        Ok(event_metrics) => {
            registry.register(Box::new(event_metrics)).unwrap();
            status_gauge.set(1);
            labels = &["success"];
        }
        Err(err) => {
            warn!("{}", err);
            labels = &["error"];
        }
    }
    registry.register(Box::new(status_gauge)).unwrap();
    let exporter_metric = exporter_metrics_family
        .get_metric_with_label_values(labels)
        .unwrap();
    exporter_metric.inc();

    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let mut metric_families = gather();
    metric_families.extend(registry.gather());
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}

lazy_static! {
    static ref HOME_PAGE: String = format!(
        "
        <html>
        <head><title>AWS Health Exporter</title></head>
        <body>
            AWS Health Exporter v{}
            <ul>
                <li><a href=\"/status\">Exporter status</a></li>
                <li><a href=\"/metrics\">Metrics</a></li>
            </ul>
        </body>
    </html>
    ",
        crate_version!()
    );
}

static STATUS_PAGE: &str =
    "<html><head><title>AWS Health Exporter</title></head><body>Ok</body></html>";
