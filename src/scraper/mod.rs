use rusoto_core::Region;
use rusoto_health::{AWSHealthClient, DescribeEventsRequest, AWSHealth, Event};
use std::str::FromStr;

mod error;
use error::Result;

pub(crate) struct Scraper {
    client: AWSHealthClient,
    regions: Option<Vec<String>>,
    locale: Option<String>,
}

impl Scraper {
    pub fn new(regions: Option<Vec<String>>) -> Self {
        let client = AWSHealthClient::new(Region::from_str("us-east-1").unwrap());
        Self { client, regions, locale: Some("en".into()) }
    }

    pub async fn describe_events(&self) -> Option<Vec<Event>> {
        let result: &mut Vec<&Event> = &mut Vec::new();

        let mut request = &mut DescribeEventsRequest{
            filter: None,
            locale: self.locale.to_owned(),
            max_results: None,
            next_token: None,
        };

        loop {
            match self.client.describe_events(*request).await {
                Ok(describe_events_response) => {
                    match describe_events_response.events {
                        Some(events) => *result.append(events),
                        None => ()
                    }
                    match describe_events_response.next_token {
                        Some(token) => request.next_token = Some(token),
                        None => break,
                    }
                },
                Err(err) => break,
            }
        }

        Some(result)
    }
}
