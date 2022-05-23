use aws_config::sts::AssumeRoleProvider;
use aws_sdk_health::Region;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::env;
use tracing::debug;

// mod exporter;
mod scraper;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let role_arn = env::var("AWS_HEALTH_EXPORTER_ROLE")?;
    let base_config = aws_config::load_from_env().await;
    let base_credentials = base_config
        .credentials_provider()
        .ok_or_else(|| eyre!("Failed to retrieve base credentials"))?;
    let base_region = base_config
        .region()
        .ok_or_else(|| eyre!("Failed to get base region"))?;
    let sts_credential_provider = AssumeRoleProvider::builder(role_arn)
        .session_name("AWS_Health_Exporter")
        .region(base_region.clone())
        .build(base_credentials.clone());

    let config = aws_config::from_env()
        .region(Region::new("us-east-1")) // AWS Health is only available from this region
        .credentials_provider(sts_credential_provider)
        .load()
        .await;
    let client = aws_sdk_health::client::Client::new(&config);

    let s = scraper::Scraper::new(client, Some(vec!["eu-west-3"]), None);

    let events = s.get_organization_events().await?;
    for event in events {
        let accounts = s.get_affected_accounts(&event).await?;
        debug!("accounts: {:?}\nevent: {:#?}", accounts, event)
    }
    // s.get_event_details(events).await?;
    // let entities = s.get_affected_entities(events).await?;
    // println!("{:#?}", entities);

    Ok(())
}
