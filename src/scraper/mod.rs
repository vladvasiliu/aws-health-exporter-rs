use aws_sdk_health::client::Client as HealthClient;
use aws_sdk_health::model::{OrganizationEvent, OrganizationEventFilter};
use color_eyre::{Report, Result};
use tokio_stream::StreamExt;

pub struct Scraper {
    client: HealthClient,
    event_filter: OrganizationEventFilter,
}

impl Scraper {
    pub fn new(
        client: aws_sdk_health::client::Client,
        regions: Option<Vec<&str>>,
        services: Option<Vec<&str>>,
    ) -> Self {
        let event_filter = OrganizationEventFilter::builder()
            .set_services(services.map(|x| x.iter().map(|s| s.to_string()).collect()))
            .set_regions(regions.map(|x| x.iter().map(|r| r.to_string()).collect()))
            .build();
        Self {
            client,
            event_filter,
        }
    }

    pub async fn get_organization_events(&self) -> Result<Vec<OrganizationEvent>> {
        let response = self
            .client
            .describe_events_for_organization()
            .set_filter(Some(self.event_filter.clone()))
            .into_paginator()
            .items()
            .send();

        response
            .collect::<Result<Vec<OrganizationEvent>, _>>()
            .await
            .map_err(Report::from)
    }
}
