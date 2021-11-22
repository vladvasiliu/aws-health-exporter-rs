use aws_sdk_health::model::OrganizationEventFilter;
use color_eyre::Result;

mod scraper;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    let config = aws_config::load_from_env().await;
    let client = aws_sdk_health::client::Client::new(&config);

    let regions = vec!["eu-west-3".into()];
    let filter = OrganizationEventFilter::builder()
        .set_regions(Some(regions))
        .build();

    let mut events = vec![];
    let mut next_token = None;
    loop {
        let response = client
            .describe_events_for_organization()
            .set_filter(Some(filter.clone()))
            .set_next_token(next_token)
            .send()
            .await?;

        if let Some(events_vec) = response.events {
            events.extend(events_vec)
        }

        next_token = response.next_token;
        if next_token.is_none() {
            break;
        }
    }

    println!("{:#?}", events);

    Ok(())
}
