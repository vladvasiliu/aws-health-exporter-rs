use std::collections::HashMap;
use std::default::Default;
use std::str::FromStr;
use std::time::Duration;

use prometheus::{opts, IntGaugeVec};
use rusoto_core::request::BufferedHttpResponse;
use rusoto_core::{HttpClient, Region, RusotoError};
use rusoto_health::{
    AWSHealth, AWSHealthClient, DescribeEventsForOrganizationRequest,
    DescribeEventsForOrganizationResponse, DescribeEventsRequest, DescribeEventsResponse, Event,
    EventFilter, OrganizationEvent, OrganizationEventFilter,
};
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use tokio::time::sleep;
use tracing::debug;
use warp::http::StatusCode;

use error::{Error, Result};

use crate::config::Config;

pub(crate) mod error;

// AWS Health API is only available on us-east-1
static HEALTH_REGION: &str = "us-east-1";

pub(crate) struct Scraper {
    client: AWSHealthClient,
    regions: Option<Vec<String>>,
    services: Option<Vec<String>>,
    locale: Option<String>,
    use_organization: bool,
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
            use_organization: config.use_organization,
        })
    }

    pub async fn describe_events(&self) -> Result<IntGaugeVec> {
        let opts = opts!("aws_health_events", "A list of AWS Health events");
        let labels = if self.use_organization {
            vec![
                "event_type_category",
                "event_type_code",
                "region",
                "service",
                "status",
            ]
        } else {
            vec![
                "availability_zone",
                "event_type_category",
                "event_type_code",
                "region",
                "service",
                "status",
            ]
        };
        let event_metrics = IntGaugeVec::new(opts, &labels)?;

        let next_token: Option<String> = None;
        let generic_filter = GenericFilter {
            regions: self.regions.to_owned(),
            services: self.services.to_owned(),
            event_type_categories: Some(vec!["issue".to_string(), "scheduledChange".to_string()]),
        };

        let mut request = GenericRequest {
            filter: Some(generic_filter),
            locale: self.locale.to_owned(),
            max_results: None,
            next_token: next_token.to_owned(),
        };

        // Implement a poor man's backoff
        // As documented on "Handling errors / Error retries and exponential backoff"
        // https://docs.aws.amazon.com/elastictranscoder/latest/developerguide/error-handling.html#api-retries
        let mut retry: u32 = 0;
        let wait_base: u32 = 2;
        loop {
            if retry > 10 {
                return Err(Error::TooManyRetries);
            }
            if retry > 0 {
                let delay = Duration::from_millis(50) * wait_base.pow(retry);
                debug!("Got TooManyRequests. Sleeping for {:#?}...", delay);
                sleep(delay).await;
            }
            let response: Box<dyn GenericResponse> = if self.use_organization {
                let request = request.clone().into();
                match self.client.describe_events_for_organization(request).await {
                    Ok(response) => Box::new(response),
                    Err(RusotoError::Unknown(BufferedHttpResponse {
                        status: StatusCode::TOO_MANY_REQUESTS,
                        ..
                    })) => {
                        retry += 1;
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                }
            } else {
                let request = request.clone().into();
                match self.client.describe_events(request).await {
                    Ok(response) => Box::new(response),
                    Err(RusotoError::Unknown(BufferedHttpResponse {
                        status: StatusCode::TOO_MANY_REQUESTS,
                        ..
                    })) => {
                        retry += 1;
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                }
            };
            response.get_events(&event_metrics)?;
            match response.get_next_token() {
                Some(token) => request.next_token = Some(token),
                None => break,
            }
            retry = 0;
        }

        Ok(event_metrics)
    }
}

fn handle_events<T: GenericEvent>(events: &[T], metric_family: &IntGaugeVec) -> Result<()> {
    for event in events {
        let metric = metric_family.get_metric_with(&event.get_fields())?;
        metric.set(1);
    }
    Ok(())
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

#[derive(Clone)]
struct GenericFilter {
    regions: Option<Vec<String>>,
    services: Option<Vec<String>>,
    event_type_categories: Option<Vec<String>>,
}

impl From<GenericFilter> for EventFilter {
    fn from(generic_filter: GenericFilter) -> Self {
        Self {
            regions: generic_filter.regions,
            services: generic_filter.services,
            event_type_categories: generic_filter.event_type_categories,
            ..Default::default()
        }
    }
}

impl From<GenericFilter> for OrganizationEventFilter {
    fn from(generic_filter: GenericFilter) -> Self {
        Self {
            regions: generic_filter.regions,
            services: generic_filter.services,
            event_type_categories: generic_filter.event_type_categories,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
struct GenericRequest {
    filter: Option<GenericFilter>,
    locale: Option<String>,
    max_results: Option<i64>,
    next_token: Option<String>,
}

impl From<GenericRequest> for DescribeEventsRequest {
    fn from(generic_request: GenericRequest) -> Self {
        Self {
            filter: generic_request.filter.map(EventFilter::from),
            locale: generic_request.locale,
            max_results: generic_request.max_results,
            next_token: generic_request.next_token,
        }
    }
}

impl From<GenericRequest> for DescribeEventsForOrganizationRequest {
    fn from(generic_request: GenericRequest) -> Self {
        Self {
            filter: generic_request.filter.map(OrganizationEventFilter::from),
            locale: generic_request.locale,
            max_results: generic_request.max_results,
            next_token: generic_request.next_token,
        }
    }
}

trait GenericResponse {
    fn get_next_token(&self) -> Option<String>;
    fn get_events(&self, metric_family: &IntGaugeVec) -> Result<()>;
}

impl GenericResponse for DescribeEventsResponse {
    fn get_next_token(&self) -> Option<String> {
        self.next_token.clone()
    }

    fn get_events(&self, metric_family: &IntGaugeVec) -> Result<()> {
        if let Some(events) = &self.events {
            handle_events(events, metric_family)?;
        }
        Ok(())
    }
}

impl GenericResponse for DescribeEventsForOrganizationResponse {
    fn get_next_token(&self) -> Option<String> {
        self.next_token.clone()
    }

    fn get_events(&self, metric_family: &IntGaugeVec) -> Result<()> {
        if let Some(events) = &self.events {
            handle_events(events, metric_family)?;
        }
        Ok(())
    }
}
