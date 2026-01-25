use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::nrf::{NfType, NrfClient};

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
    nrf_client: Option<Arc<NrfClient>>,
    udm_uri_cache: Arc<RwLock<Option<String>>>,
    fallback_udm_uri: Option<String>,
}

impl UdmClient {
    pub fn new(udm_uri: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            nrf_client: None,
            udm_uri_cache: Arc::new(RwLock::new(None)),
            fallback_udm_uri: Some(udm_uri),
        }
    }

    pub fn with_nrf(nrf_client: Arc<NrfClient>, fallback_udm_uri: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            nrf_client: Some(nrf_client),
            udm_uri_cache: Arc::new(RwLock::new(None)),
            fallback_udm_uri,
        }
    }

    async fn get_udm_uri(&self, supi: &str) -> Result<String> {
        if let Some(ref nrf_client) = self.nrf_client {
            {
                let cached = self.udm_uri_cache.read().await;
                if let Some(ref uri) = *cached {
                    return Ok(uri.clone());
                }
            }

            match self.discover_udm(nrf_client, supi).await {
                Ok(uri) => {
                    let mut cache = self.udm_uri_cache.write().await;
                    *cache = Some(uri.clone());
                    Ok(uri)
                }
                Err(e) => {
                    warn!("Failed to discover UDM via NRF: {}", e);
                    if let Some(ref fallback) = self.fallback_udm_uri {
                        info!("Using fallback UDM URI: {}", fallback);
                        Ok(fallback.clone())
                    } else {
                        Err(e)
                    }
                }
            }
        } else if let Some(ref fallback) = self.fallback_udm_uri {
            Ok(fallback.clone())
        } else {
            Err(anyhow::anyhow!("No NRF client or fallback UDM URI configured"))
        }
    }

    async fn discover_udm(&self, nrf_client: &NrfClient, supi: &str) -> Result<String> {
        let mut query_params = std::collections::HashMap::new();
        query_params.insert("supi".to_string(), supi.to_string());

        let search_result = nrf_client
            .discover(NfType::Udm, Some(query_params))
            .await
            .context("Failed to discover UDM from NRF")?;

        if search_result.nf_instances.is_empty() {
            return Err(anyhow::anyhow!("No UDM instances found"));
        }

        let udm_instance = search_result
            .nf_instances
            .iter()
            .max_by_key(|inst| {
                (
                    inst.priority.unwrap_or(u16::MAX),
                    std::cmp::Reverse(inst.capacity.unwrap_or(0)),
                    std::cmp::Reverse(inst.load.unwrap_or(u16::MAX)),
                )
            })
            .ok_or_else(|| anyhow::anyhow!("No valid UDM instance found"))?;

        if let Some(ref services) = udm_instance.nf_services {
            for service in services {
                if service.service_name == "nudm-sdm" {
                    if let Some(ref api_prefix) = service.api_prefix {
                        info!("Discovered UDM at {}", api_prefix);
                        return Ok(api_prefix.clone());
                    }
                }
            }
        }

        if let Some(ref ipv4_addrs) = udm_instance.ipv4_addresses {
            if let Some(first_addr) = ipv4_addrs.first() {
                let udm_uri = format!("http://{}", first_addr);
                info!("Discovered UDM at {}", udm_uri);
                return Ok(udm_uri);
            }
        }

        Err(anyhow::anyhow!("No valid UDM endpoint found"))
    }

    pub async fn get_am_data(&self, supi: &str) -> Result<AccessAndMobilitySubscriptionData> {
        let udm_uri = self.get_udm_uri(supi).await?;
        let url = format!("{}/nudm-sdm/v2/{}/am-data", udm_uri, supi);

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
        let udm_uri = self.get_udm_uri(supi).await?;
        let url = format!("{}/nudm-sdm/v2/{}/sms-data", udm_uri, supi);

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
        let udm_uri = self.get_udm_uri(supi).await?;
        let url = format!("{}/nudm-sdm/v2/{}/sms-mng-data", udm_uri, supi);

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
