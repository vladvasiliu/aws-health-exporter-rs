use aws_config::sts::AssumeRoleProvider;
use aws_config::{BehaviorVersion, Region};
use std::env;
use anyhow::Result;
use tracing_subscriber::prelude::*;
use tracing_error::ErrorLayer;
use crate::exporter::{Server};


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

    let listen_address = env::var("AWS_HEALTH_EXPORTER_LISTEN").unwrap_or_else(|_| "[::]:9679".to_string());
    

    let role_arn = env::var("AWS_HEALTH_EXPORTER_ROLE")?;
    let sts_credential_provider = AssumeRoleProvider::builder(role_arn)
        .session_name("AWS_Health_Exporter")
        .build().await;
    
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let health_config = aws_sdk_health::config::Builder::from(&sdk_config).credentials_provider(sts_credential_provider).region(Region::new("us-east-1")).build();
    let client = aws_sdk_health::client::Client::from_conf(health_config);
    
    let s = scraper::Scraper::new(client, Some(vec!["eu-west-3"]), None);
    
    let server = Server::new(listen_address, s);
    server.serve().await?;
    
    Ok(())
}
