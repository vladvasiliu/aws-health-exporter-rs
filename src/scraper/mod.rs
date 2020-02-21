use prometheus::{opts, IntGaugeVec};
use rusoto_core::Region;
use rusoto_health::{AWSHealth, AWSHealthClient, DescribeEventsRequest, Event, EventFilter};
use std::collections::HashMap;
use std::default::Default;
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
        // AWS Health API is only available on us-east-1
        let client = AWSHealthClient::new(Region::from_str("us-east-1").unwrap());

        Self {
            client,
            regions,
            locale: Some("en".into()),
        }
    }

    pub async fn describe_events(&self) -> Result<IntGaugeVec> {
        let opts = opts!("aws_health_events", "A list of AWS Health events");
        let labels = [
            "availability_zone",
            "event_type_category",
            "event_type_code",
            "region",
            "service",
            "status",
        ];
        let event_metrics = IntGaugeVec::new(opts, &labels)?;

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

            let describe_events_response = self.client.describe_events(request).await?;
            if let Some(events) = describe_events_response.events {
                self.handle_events(events, &event_metrics);
            }
            match describe_events_response.next_token {
                Some(token) => next_token = Some(token),
                None => break,
            }
        }

        Ok(event_metrics)
    }

    fn handle_events(&self, events: Vec<Event>, metric_family: &IntGaugeVec) {
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

            let metric = metric_family.get_metric_with(&label_map).unwrap();
            metric.set(1);
        }
    }
}
