use aws_config::sts::AssumeRoleProvider;
use aws_config::{BehaviorVersion, Region};
use std::env;
use tracing::info;
use anyhow::Result;
use tracing_subscriber::prelude::*;
use tracing_error::ErrorLayer;
use crate::exporter::Server;


// mod exporter;
mod scraper;
mod exporter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .json()
        .finish()
        .with(ErrorLayer::default())
        .init();

    let server = Server::new("[::]:3000");
    server.serve().await?;

    // let role_arn = env::var("AWS_HEALTH_EXPORTER_ROLE")?;
    // let sts_credential_provider = AssumeRoleProvider::builder(role_arn)
    //     .session_name("AWS_Health_Exporter")
    //     .build().await;
    // 
    // let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    // let health_config = aws_sdk_health::config::Builder::from(&sdk_config).credentials_provider(sts_credential_provider).region(Region::new("us-east-1")).build();
    // let client = aws_sdk_health::client::Client::from_conf(health_config);
    // 
    // let s = scraper::Scraper::new(client, Some(vec!["eu-west-3"]), None);
    // 
    // let events = s.get_organization_events().await?;
    // info!("Got {} events", events.len());
    // for event in events {
    //     let accounts = s.get_affected_accounts(&event).await?;
    //     info!("accounts: {:?}\nevent: {:#?}", accounts, event)
    // }
    
    // s.get_event_details(events).await?;
    // let entities = s.get_affected_entities(events).await?;
    // println!("{:#?}", entities);

    Ok(())
}
