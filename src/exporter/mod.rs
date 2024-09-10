use std::sync::Arc;
use prometheus_client::encoding::text::encode;

use anyhow::{anyhow, Context, Result};
use aws_sdk_health::types::{OrganizationEvent};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::registry::Registry;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::family::Family;
use tokio::signal;
use tracing::{debug, warn};
use crate::scraper::Scraper;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Server {
    address: String,
    main_registry: Registry,
    scraper: Arc<Scraper>,
}

impl Server {
    pub fn new(address: String, scraper: Scraper) -> Self {
        let main_registry = <Registry>::default();
        let scraper = Arc::new(scraper);

        Self { address, main_registry, scraper }
    }

    pub async fn serve(&self) -> Result<()> {
        let scraper = self.scraper.clone();
        let app = Router::new().route("/", get(|| async { format!("AWS Health Exporter v{}", VERSION) })).route("/metrics", get(|| async {

            let result = scrape(scraper).await.context("failed to scrape events").and_then(|registry| {
                let mut buffer = String::new();

                encode(&mut buffer, &registry).context("failed to encode registry")?;
                Ok(buffer)
            });

            match result {
                Ok(r) => (StatusCode::OK, r),
                Err(err) => {
                    warn!(error = err.root_cause(), "failed to scrape events");
                    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
                }
            }
        }));

        let listener = tokio::net::TcpListener::bind(&self.address).await?;

        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await.unwrap();

        Ok(())
    }
}

pub async fn scrape(scraper: Arc<Scraper>) -> Result<Registry> {
    let mut registry = <Registry>::default();

    let health_events = Family::<HealthLabels, Gauge>::default();


    let events = scraper.get_organization_events().await?;
    debug!("Got {} events", events.len());

    for event in events {
        let accounts = scraper.get_affected_accounts(&event).await?;
        for account in accounts {
            let labels = HealthLabels::new(&account, &event)?;
            debug!("{:?}", labels);
            health_events.get_or_create(&labels).set(1);
        }
    }
    registry.register("aws_health_event", "Status of AWS Health events", health_events);

    Ok(registry)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
        let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}


#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct HealthLabels {
    status_code: String,
    region: String,
    event_type_code: String,
    service: String,
    account: String
}

impl HealthLabels {
    pub fn new(account: &str, oe: &OrganizationEvent) -> Result<Self> {
        Ok(Self {
            status_code: oe.status_code.as_ref().ok_or_else(|| anyhow!("Missing status code"))?.to_string(),
            region: oe.region.clone().ok_or_else(|| anyhow!("Missing region"))?,
            event_type_code: oe.event_type_code.clone().ok_or_else(|| anyhow!("Missing region"))?,
            service: oe.service.clone().ok_or_else(|| anyhow!("Missing region"))?,
            account: account.to_string()
        })
    }
}
