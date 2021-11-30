use crate::scraper::STSCredentialsProvider;
use aws_sdk_health::Region;
use color_eyre::Result;
use std::env;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::info;

mod scraper;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let role_arn = env::var("AWS_HEALTH_EXPORTER_ROLE")?;

    let credential_provider = STSCredentialsProvider::new(&role_arn, None, None);
    let config = aws_config::from_env()
        .region(Region::new("us-east-1"))
        .credentials_provider(credential_provider)
        .load()
        .await;
    let client = aws_sdk_health::client::Client::new(&config);

    let s = scraper::Scraper::new(client, Some(vec!["eu-west-3"]), None);
    let d = Duration::from_secs(30);
    let mut i = interval(d);
    i.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut count = 0;
    loop {
        i.tick().await;
        count += 1;
        let res = match s.get_organization_events().await {
            Ok(_) => "OK".to_string(),
            Err(e) => format!("{:?}", e),
        };
        info!(
            "Run #{} (t+{}s): {}",
            count,
            count * i.period().as_secs(),
            res
        );
    }

    Ok(())
}
