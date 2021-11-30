use aws_sdk_health::client::Client as HealthClient;
use aws_sdk_health::model::{OrganizationEvent, OrganizationEventFilter};
use aws_types::credentials::future::ProvideCredentials;
use aws_types::credentials::CredentialsError;
use aws_types::{credentials, Credentials};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use std::fmt::{Debug, Formatter};
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::debug;

// const STS_CREDENTIAL_DURATION: i32 = 900; // How long should the temporary credentials live
const STS_CREDENTIAL_CACHE_TIMEOUT: i32 = 60; // Minimum time left before refreshing credentials
const STS_SESSION_NAME: &str = "aws_health_exporter";

/// Returns whether the stored credentials are valid
/// The credentials are valid iff
/// * they're not None
/// * they expire at least `STS_CREDENTIAL_DURATION` seconds in the future
fn validate_stored_credentials(creds: Option<&Credentials>) -> Option<&Credentials> {
    if let Some(c) = creds {
        let min_expiry_time =
            SystemTime::now() + Duration::from_secs(STS_CREDENTIAL_CACHE_TIMEOUT as u64);
        let expiration = c.expiry().unwrap();
        if expiration > min_expiry_time {
            return creds;
        }
    }
    None
}

#[derive(Debug)]
pub struct STSCredentialsProvider {
    role_arn: String,
    external_id: Option<String>,
    source_identity: Option<String>,
    cred_cache: RwLock<Option<Credentials>>,
}

impl STSCredentialsProvider {
    pub fn new(role_arn: &str, external_id: Option<&str>, source_identity: Option<&str>) -> Self {
        Self {
            role_arn: role_arn.to_string(),
            external_id: external_id.map(String::from),
            source_identity: source_identity.map(String::from),
            cred_cache: RwLock::new(None),
        }
    }

    async fn load_credentials(&self) -> aws_types::credentials::Result {
        debug!("load_credentials called");
        let sts_config = aws_config::load_from_env().await;
        let sts_client = aws_sdk_sts::client::Client::new(&sts_config);
        sts_client
            .assume_role()
            .role_arn(&self.role_arn)
            .role_session_name(STS_SESSION_NAME)
            .set_external_id(self.external_id.clone())
            .set_source_identity(self.source_identity.clone())
            // .duration_seconds(STS_CREDENTIAL_DURATION)
            .send()
            .await
            .map_err(|e| CredentialsError::provider_error(e))?
            .credentials
            .map(|c| {
                credentials::Credentials::new(
                    c.access_key_id.unwrap(),
                    c.secret_access_key.unwrap(),
                    c.session_token,
                    c.expiration.map(|e| e.try_into().unwrap()),
                    "STSCredentialsProvider",
                )
            })
            .ok_or(CredentialsError::not_loaded(
                "STS Assume Role returned no credentials".to_string(),
            ))
    }

    /// Returns the credentials from caches or updates the cache if they're expired
    async fn get_credentials(&self) -> aws_types::credentials::Result {
        let mut lock = self.cred_cache.write().await;
        match validate_stored_credentials(lock.as_ref()) {
            Some(creds) => {
                debug!("Returning cached credentials");
                Ok(creds.clone())
            }
            None => {
                debug!("No valid credentials in cache. Getting from STS");
                let new_creds = self.load_credentials().await;
                match &new_creds {
                    Ok(creds) => *lock = Some(creds.clone()),
                    Err(_) => *lock = None,
                };
                new_creds
            }
        }
    }
}

impl credentials::ProvideCredentials for STSCredentialsProvider {
    fn provide_credentials<'a>(&'a self) -> ProvideCredentials<'a>
    where
        Self: 'a,
    {
        aws_types::credentials::future::ProvideCredentials::new(self.get_credentials())
    }
}

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
        let mut events = vec![];
        let mut next_token = None;

        loop {
            let response = self
                .client
                .describe_events_for_organization()
                .set_filter(Some(self.event_filter.clone()))
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

        Ok(events)
    }
}
