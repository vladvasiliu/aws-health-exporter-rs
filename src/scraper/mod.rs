use prometheus::{opts, IntGaugeVec};
use rusoto_core::{HttpClient, Region};
use rusoto_health::{
    AWSHealth, AWSHealthClient, DescribeEventsForOrganizationRequest, DescribeEventsRequest, Event,
    EventFilter, OrganizationEvent, OrganizationEventFilter,
};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use std::collections::HashMap;
use std::default::Default;
use std::str::FromStr;

pub(crate) mod error;
use crate::config::Config;
use error::Result;

// AWS Health API is only available on us-east-1
static HEALTH_REGION: &str = "us-east-1";

pub(crate) struct Scraper {
    client: AWSHealthClient,
    regions: Option<Vec<String>>,
    services: Option<Vec<String>>,
    locale: Option<String>,
}

impl Scraper {
    /// Create a new scraper and handle authentication.
    ///
    /// Documentation related to handling assumed roles:
    /// https://github.com/rusoto/rusoto/blob/master/AWS-CREDENTIALS.md
    pub fn new(config: &Config) -> Result<Self> {
        let health_region = Region::from_str(HEALTH_REGION)?;

        let client = match &config.role {
            None => AWSHealthClient::new(health_region),
            Some(role) => {
                let sts_region = match &config.role_region {
                    Some(region) => Region::from_str(region)?,
                    None => Region::default(),
                };
                let sts = StsClient::new(sts_region);

                let sts_provider = StsAssumeRoleSessionCredentialsProvider::new(
                    sts,
                    role.to_owned(),
                    "aws-health-exporter".to_owned(),
                    None,
                    None,
                    None,
                    None,
                );
                let auto_refreshing_provider =
                    rusoto_credential::AutoRefreshingProvider::new(sts_provider)?;
                AWSHealthClient::new_with(
                    HttpClient::new()?,
                    auto_refreshing_provider,
                    health_region,
                )
            }
        };

        Ok(Self {
            client,
            regions: config.regions.to_owned(),
            locale: Some("en".into()),
            services: config.services.to_owned(),
        })
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
            services: self.services.to_owned(),
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
                self.handle_events(events, &event_metrics)?;
            }
            match describe_events_response.next_token {
                Some(token) => next_token = Some(token),
                None => break,
            }
        }

        Ok(event_metrics)
    }

    pub async fn describe_organization_events(&self) -> Result<IntGaugeVec> {
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
        let filter = Some(OrganizationEventFilter {
            regions: self.regions.to_owned(),
            services: self.services.to_owned(),
            event_type_categories: Some(vec!["issue".to_string(), "scheduledChange".to_string()]),
            ..Default::default()
        });

        loop {
            let request = DescribeEventsForOrganizationRequest {
                filter: filter.to_owned(),
                locale: self.locale.to_owned(),
                max_results: None,
                next_token: next_token.to_owned(),
            };

            let describe_events_response = self
                .client
                .describe_events_for_organization(request)
                .await
                .unwrap();
            if let Some(events) = describe_events_response.events {
                self.handle_events(events, &event_metrics)?;
            }
            match describe_events_response.next_token {
                Some(token) => next_token = Some(token),
                None => break,
            }
        }

        Ok(event_metrics)
    }

    fn handle_events<T: GenericEvent>(
        &self,
        events: Vec<T>,
        metric_family: &IntGaugeVec,
    ) -> Result<()> {
        for event in events {
            let metric = metric_family.get_metric_with(&event.get_fields())?;
            metric.set(1);
        }
        Ok(())
    }
}

trait GenericEvent {
    fn get_fields(&self) -> HashMap<&str, &str>;
}

impl GenericEvent for Event {
    fn get_fields(&self) -> HashMap<&str, &str> {
        let mut label_map: HashMap<&str, &str> = HashMap::new();

        let availability_zone = self.availability_zone.as_ref().map_or("", String::as_str);
        let region = self.region.as_ref().map_or("", String::as_str);
        let service = self.service.as_ref().map_or("", String::as_str);
        let event_type_category = self.event_type_category.as_ref().map_or("", String::as_str);
        let event_type_code = self.event_type_code.as_ref().map_or("", String::as_str);
        let status = self.status_code.as_ref().map_or("", String::as_str);

        label_map.insert("availability_zone", availability_zone);
        label_map.insert("event_type_category", event_type_category);
        label_map.insert("event_type_code", event_type_code);
        label_map.insert("region", region);
        label_map.insert("service", service);
        label_map.insert("status", status);

        label_map
    }
}

impl GenericEvent for OrganizationEvent {
    fn get_fields(&self) -> HashMap<&str, &str> {
        let mut label_map: HashMap<&str, &str> = HashMap::new();

        let region = self.region.as_ref().map_or("", String::as_str);
        let service = self.service.as_ref().map_or("", String::as_str);
        let event_type_category = self.event_type_category.as_ref().map_or("", String::as_str);
        let event_type_code = self.event_type_code.as_ref().map_or("", String::as_str);
        let status = self.status_code.as_ref().map_or("", String::as_str);

        label_map.insert("event_type_category", event_type_category);
        label_map.insert("event_type_code", event_type_code);
        label_map.insert("region", region);
        label_map.insert("service", service);
        label_map.insert("status", status);

        label_map
    }
}
