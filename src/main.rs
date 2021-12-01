use aws_sdk_health::Region;
use aws_sdk_sts_caching_provider::STSCredentialsProvider;
use color_eyre::Result;
use std::env;

// mod exporter;
mod scraper;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let role_arn = env::var("AWS_HEALTH_EXPORTER_ROLE")?;

    let credential_provider =
        STSCredentialsProvider::new(&role_arn, None, None, Some("aws_health_exporter"), None, 60);
    let config = aws_config::from_env()
        .region(Region::new("us-east-1"))
        .credentials_provider(credential_provider)
        .load()
        .await;
    let client = aws_sdk_health::client::Client::new(&config);

    let s = scraper::Scraper::new(client, Some(vec!["eu-west-3"]), None);

    let events = s.get_organization_events().await?;
    println!("{:?}", events);

    Ok(())
}
