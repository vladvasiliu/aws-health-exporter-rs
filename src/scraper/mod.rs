use rusoto_core::Region;
use rusoto_health::{AWSHealth, AWSHealthClient, DescribeEventsRequest, Event, EventFilter};
use std::default::Default;
use std::str::FromStr;

use prometheus::{opts, register, GaugeVec};
use std::collections::HashMap;
use std::f64::NAN;

mod error;

pub(crate) struct Scraper {
    client: AWSHealthClient,
    regions: Option<Vec<String>>,
    locale: Option<String>,
    start_time_metric: GaugeVec,
    end_time_metric: GaugeVec,
    last_updated_time_metric: GaugeVec,
}

impl Scraper {
    pub fn new(regions: Option<Vec<String>>) -> Self {
        // AWS Health API is only available on us-east-1
        let client = AWSHealthClient::new(Region::from_str("us-east-1").unwrap());

        let labels = [
            "availability_zone",
            "event_type_category",
            "region",
            "service",
            "status",
        ];
        let start_time_opts = opts!("event_start_time", "Event start time");
        let start_time_metric = GaugeVec::new(start_time_opts, &labels).unwrap();
        register(Box::new(start_time_metric.clone())).unwrap();

        let end_time_opts = opts!("event_end_time", "Event end time");
        let end_time_metric = GaugeVec::new(end_time_opts, &labels).unwrap();
        register(Box::new(end_time_metric.clone())).unwrap();

        let last_updated_time_opts = opts!("event_last_updated_time", "Event last_updated time");
        let last_updated_time_metric = GaugeVec::new(last_updated_time_opts, &labels).unwrap();
        register(Box::new(last_updated_time_metric.clone())).unwrap();

        Self {
            client,
            regions,
            locale: Some("en".into()),
            start_time_metric,
            end_time_metric,
            last_updated_time_metric,
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
            let status = event.status_code.unwrap_or_default();

            label_map.insert("availability_zone", &availability_zone);
            label_map.insert("event_type_category", &event_type_category);
            label_map.insert("region", &region);
            label_map.insert("service", &service);
            label_map.insert("status", &status);

            let start = self.start_time_metric.get_metric_with(&label_map).unwrap();
            start.set(event.start_time.unwrap_or(NAN));

            let end = self.end_time_metric.get_metric_with(&label_map).unwrap();
            end.set(event.end_time.unwrap_or(NAN));

            let last = self
                .last_updated_time_metric
                .get_metric_with(&label_map)
                .unwrap();
            last.set(event.last_updated_time.unwrap_or(NAN));
        }
    }
}
