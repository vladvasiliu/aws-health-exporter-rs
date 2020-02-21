use rusoto_core::Region;
use rusoto_health::{AWSHealth, AWSHealthClient, DescribeEventsRequest, Event, EventFilter};
use std::default::Default;
use std::str::FromStr;

use prometheus::{opts, register, GaugeVec};
use std::collections::HashMap;

mod error;

pub(crate) struct Scraper {
    client: AWSHealthClient,
    regions: Option<Vec<String>>,
    locale: Option<String>,
    event_metrics: GaugeVec,
}

impl Scraper {
    pub fn new(regions: Option<Vec<String>>) -> Self {
        // AWS Health API is only available on us-east-1
        let client = AWSHealthClient::new(Region::from_str("us-east-1").unwrap());

        let labels = [
            "availability_zone",
            "event_type_category",
            "event_type_code",
            "region",
            "service",
            "status",
        ];

        let opts = opts!("aws_health_events", "A list of AWS Health events");
        let event_metrics = GaugeVec::new(opts, &labels).unwrap();
        register(Box::new(event_metrics.clone())).unwrap();

        Self {
            client,
            regions,
            locale: Some("en".into()),
            event_metrics,
        }
    }

    pub async fn describe_events(&self) {
        let mut next_token: Option<String> = None;
        let filter = Some(EventFilter {
            regions: self.regions.to_owned(),
            event_type_categories: Some(vec!["issue".to_string(), "scheduledChange".to_string()]),
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
                        self.handle_events(events);
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

    fn handle_events(&self, events: Vec<Event>) {
        for event in events {
            let mut label_map: HashMap<&str, &str> = HashMap::new();

            let availability_zone = event.availability_zone.unwrap_or_default();
            let region = event.region.unwrap_or_default();
            let service = event.service.unwrap_or_default();
            let event_type_category = event.event_type_category.unwrap_or_default();
            let event_type_code = event.event_type_code.unwrap_or_default();
            let status = event.status_code.unwrap_or_default();

            label_map.insert("availability_zone", &availability_zone);
            label_map.insert("event_type_category", &event_type_category);
            label_map.insert("event_type_code", &event_type_code);
            label_map.insert("region", &region);
            label_map.insert("service", &service);
            label_map.insert("status", &status);

            let metric = self.event_metrics.get_metric_with(&label_map).unwrap();
            metric.set(1.0);
        }
    }
}
