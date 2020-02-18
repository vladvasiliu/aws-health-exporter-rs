use rusoto_core::Region;
use rusoto_health::{AWSHealth, AWSHealthClient, DescribeEventsRequest, Event, EventFilter};
use std::default::Default;
use std::str::FromStr;

mod error;

pub(crate) struct Scraper {
    client: AWSHealthClient,
    regions: Option<Vec<String>>,
    locale: Option<String>,
}

impl Scraper {
    pub fn new(regions: Option<Vec<String>>) -> Self {
        // AWS Health API is only available on us-east-1
        let client = AWSHealthClient::new(Region::from_str("us-east-1").unwrap());
        Self {
            client,
            regions,
            locale: Some("en".into()),
        }
    }

    pub async fn describe_events(&self) {
        let mut next_token: Option<String> = None;
        let filter = Some(EventFilter {
            regions: self.regions.to_owned(),
            ..Default::default()
        });

        loop {
            let request = DescribeEventsRequest {
                filter: filter.to_owned(),
                locale: self.locale.to_owned(),
                max_results: None,
                next_token: next_token.to_owned(),
            };

            match self.client.describe_events(request).await {
                Ok(describe_events_response) => {
                    if let Some(events) = describe_events_response.events {
                        handle_events(events);
                    }
                    if let Some(token) = describe_events_response.next_token {
                        next_token = Some(token);
                        continue;
                    }
                }
                Err(err) => println!("Got error: {}", err),
            }
            break;
        }
    }
}

fn handle_events(events: Vec<Event>) {
    println!("{:#?}", events)
}
