use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessAndMobilitySubscriptionData {
    pub gpsis: Option<Vec<String>>,
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsSubscriptionData {
    pub sms_subscribed: Option<bool>,
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmsManagementSubscriptionData {
    pub mt_sms_subscribed: Option<bool>,
    pub mt_sms_barring_all: Option<bool>,
    pub mt_sms_barring_roaming: Option<bool>,
    pub mo_sms_subscribed: Option<bool>,
    pub mo_sms_barring_all: Option<bool>,
    pub mo_sms_barring_roaming: Option<bool>,
    pub supported_features: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SmsAuthorizationData {
    pub msisdn: Option<String>,
    pub sms_subscribed: bool,
    pub mt_sms_allowed: bool,
    pub mo_sms_allowed: bool,
    pub mt_sms_barring_roaming: bool,
    pub mo_sms_barring_roaming: bool,
}

#[derive(Clone)]
pub struct UdmClient {
    client: Client,
    udm_uri: String,
}

impl UdmClient {
    pub fn new(udm_uri: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            udm_uri,
        }
    }

    pub async fn get_am_data(&self, supi: &str) -> Result<AccessAndMobilitySubscriptionData> {
        let url = format!("{}/nudm-sdm/v2/{}/am-data", self.udm_uri, supi);

        debug!("Fetching AM data for SUPI: {}", supi);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send AM data request to UDM")?;

        match response.status() {
            StatusCode::OK => {
                let am_data = response
                    .json::<AccessAndMobilitySubscriptionData>()
                    .await
                    .context("Failed to parse AM data response")?;
                info!("Retrieved AM data for SUPI: {}", supi);
                Ok(am_data)
            }
            StatusCode::NOT_FOUND => {
                Err(anyhow::anyhow!("AM data not found for SUPI: {}", supi))
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "Failed to get AM data with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn get_sms_data(&self, supi: &str) -> Result<SmsSubscriptionData> {
        let url = format!("{}/nudm-sdm/v2/{}/sms-data", self.udm_uri, supi);

        debug!("Fetching SMS subscription data for SUPI: {}", supi);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send SMS data request to UDM")?;

        match response.status() {
            StatusCode::OK => {
                let sms_data = response
                    .json::<SmsSubscriptionData>()
                    .await
                    .context("Failed to parse SMS data response")?;
                info!("Retrieved SMS subscription data for SUPI: {}", supi);
                Ok(sms_data)
            }
            StatusCode::NOT_FOUND => {
                Err(anyhow::anyhow!("SMS data not found for SUPI: {}", supi))
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "Failed to get SMS data with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn get_sms_mng_data(
        &self,
        supi: &str,
    ) -> Result<SmsManagementSubscriptionData> {
        let url = format!("{}/nudm-sdm/v2/{}/sms-mng-data", self.udm_uri, supi);

        debug!("Fetching SMS management data for SUPI: {}", supi);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to send SMS management data request to UDM")?;

        match response.status() {
            StatusCode::OK => {
                let sms_mng_data = response
                    .json::<SmsManagementSubscriptionData>()
                    .await
                    .context("Failed to parse SMS management data response")?;
                info!("Retrieved SMS management data for SUPI: {}", supi);
                Ok(sms_mng_data)
            }
            StatusCode::NOT_FOUND => {
                Err(anyhow::anyhow!(
                    "SMS management data not found for SUPI: {}",
                    supi
                ))
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "Failed to get SMS management data with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn get_sms_authorization(&self, supi: &str) -> Result<SmsAuthorizationData> {
        let am_data_result = self.get_am_data(supi).await;
        let sms_data_result = self.get_sms_data(supi).await;
        let sms_mng_data_result = self.get_sms_mng_data(supi).await;

        let msisdn = am_data_result
            .ok()
            .and_then(|am| am.gpsis)
            .and_then(|gpsis| gpsis.first().cloned());

        let sms_subscribed = sms_data_result
            .ok()
            .and_then(|sms| sms.sms_subscribed)
            .unwrap_or(false);

        let sms_mng_data = sms_mng_data_result.ok();

        let mt_sms_subscribed = sms_mng_data
            .as_ref()
            .and_then(|mng| mng.mt_sms_subscribed)
            .unwrap_or(true);

        let mt_sms_barring_all = sms_mng_data
            .as_ref()
            .and_then(|mng| mng.mt_sms_barring_all)
            .unwrap_or(false);

        let mt_sms_barring_roaming = sms_mng_data
            .as_ref()
            .and_then(|mng| mng.mt_sms_barring_roaming)
            .unwrap_or(false);

        let mo_sms_subscribed = sms_mng_data
            .as_ref()
            .and_then(|mng| mng.mo_sms_subscribed)
            .unwrap_or(true);

        let mo_sms_barring_all = sms_mng_data
            .as_ref()
            .and_then(|mng| mng.mo_sms_barring_all)
            .unwrap_or(false);

        let mo_sms_barring_roaming = sms_mng_data
            .as_ref()
            .and_then(|mng| mng.mo_sms_barring_roaming)
            .unwrap_or(false);

        let mt_sms_allowed = sms_subscribed && mt_sms_subscribed && !mt_sms_barring_all;
        let mo_sms_allowed = sms_subscribed && mo_sms_subscribed && !mo_sms_barring_all;

        if !sms_subscribed {
            warn!("SUPI {} is not subscribed to SMS service", supi);
        }

        if mt_sms_barring_all {
            warn!("SUPI {} has MT-SMS barring enabled", supi);
        }

        if mo_sms_barring_all {
            warn!("SUPI {} has MO-SMS barring enabled", supi);
        }

        Ok(SmsAuthorizationData {
            msisdn,
            sms_subscribed,
            mt_sms_allowed,
            mo_sms_allowed,
            mt_sms_barring_roaming,
            mo_sms_barring_roaming,
        })
    }
}
