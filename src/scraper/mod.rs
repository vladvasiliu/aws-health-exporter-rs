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

    pub async fn get_affected_accounts(&self, event: &OrganizationEvent) -> Result<Vec<String>> {
        let response = self
            .client
            .describe_affected_accounts_for_organization()
            .set_event_arn(event.arn.clone())
            .into_paginator()
            .items()
            .send();

        response
            .collect::<Result<Vec<String>, _>>()
            .await
            .map_err(Report::from)
    }

    // TODO: Find a way to test those

    // pub async fn get_event_details(
    //     &self,
    //     events_vec: Vec<OrganizationEvent>,
    // ) -> Result<Vec<EventDetails>> {
    //     let details_vec = vec![];
    //
    //     for event in events_vec {
    //         let filter = self.get_event_account_filter(&event).await?;
    //         let response = self
    //             .client
    //             .describe_event_details_for_organization()
    //             .set_organization_event_detail_filters(Some(filter))
    //             .send()
    //             .await?;
    //         debug!("event details: {:#?}", response);
    //     }
    //
    //     Ok(details_vec)
    // }

    // pub async fn get_affected_entities(
    //     &self,
    //     events_vec: Vec<OrganizationEvent>,
    // ) -> Result<Vec<AffectedEntity>> {
    //     let mut entity_vec = vec![];
    //
    //     for event in events_vec {
    //         let filter = self.get_event_account_filter(&event).await?;
    //         let mut response = self
    //             .client
    //             .describe_affected_entities_for_organization()
    //             .set_organization_entity_filters(Some(filter))
    //             .into_paginator()
    //             // .items()
    //             .send();
    //         while let Some(entities) = response.next().await {
    //             debug!("Entities: {:#?}", entities);
    //         }
    //
    //         // entity_vec.extend(result);
    //     }
    //
    //     Ok(entity_vec)
    // }
    //
    // async fn get_event_account_filter(
    //     &self,
    //     event: &OrganizationEvent,
    // ) -> Result<Vec<EventAccountFilter>> {
    //     let mut result = vec![];
    //     let accounts = if event.event_scope_code == Some(EventScopeCode::AccountSpecific) {
    //         self.get_affected_accounts(&event)
    //             .await?
    //             .into_iter()
    //             .map(Option::from)
    //             .collect()
    //     } else {
    //         vec![None]
    //     };
    //     for affected_account in accounts {
    //         let filter = EventAccountFilter::builder()
    //             .set_event_arn(event.arn.clone())
    //             .set_aws_account_id(affected_account)
    //             .build();
    //         result.push(filter);
    //     }
    //     Ok(result)
    // }
}
