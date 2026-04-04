use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{error, info};

use crate::config::TlsConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NfType {
    Nrf,
    Udm,
    Amf,
    Smf,
    Ausf,
    Nef,
    Pcf,
    Smsf,
    Nssf,
    Udr,
    Lmf,
    Gmlc,
    #[serde(rename = "5G_EIR")]
    FiveGEir,
    Sepp,
    Upf,
    N3iwf,
    Af,
    Udsf,
    Bsf,
    Chf,
    Nwdaf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NfStatus {
    Registered,
    Suspended,
    Undiscoverable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NfServiceStatus {
    Registered,
    Suspended,
    Undiscoverable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlmnId {
    pub mcc: String,
    pub mnc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFProfile {
    pub nf_instance_id: String,
    pub nf_type: NfType,
    pub nf_status: NfStatus,
    pub plmn_list: Vec<PlmnId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fqdn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nf_types: Option<Vec<NfType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_services: Option<Vec<NFService>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heart_beat_timer: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFService {
    pub service_instance_id: String,
    pub service_name: String,
    pub versions: Vec<NFServiceVersion>,
    pub scheme: String,
    pub nf_service_status: NfServiceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_nf_types: Option<Vec<NfType>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NFServiceVersion {
    pub api_version_in_uri: String,
    pub api_full_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub validity_period: Option<u32>,
    pub nf_instances: Vec<NFProfile>,
    pub search_id: Option<String>,
    pub num_nf_inst_complete: Option<u32>,
}

impl std::fmt::Display for NfType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NfType::Nrf => write!(f, "NRF"),
            NfType::Udm => write!(f, "UDM"),
            NfType::Amf => write!(f, "AMF"),
            NfType::Smf => write!(f, "SMF"),
            NfType::Ausf => write!(f, "AUSF"),
            NfType::Nef => write!(f, "NEF"),
            NfType::Pcf => write!(f, "PCF"),
            NfType::Smsf => write!(f, "SMSF"),
            NfType::Nssf => write!(f, "NSSF"),
            NfType::Udr => write!(f, "UDR"),
            NfType::Lmf => write!(f, "LMF"),
            NfType::Gmlc => write!(f, "GMLC"),
            NfType::FiveGEir => write!(f, "5G_EIR"),
            NfType::Sepp => write!(f, "SEPP"),
            NfType::Upf => write!(f, "UPF"),
            NfType::N3iwf => write!(f, "N3IWF"),
            NfType::Af => write!(f, "AF"),
            NfType::Udsf => write!(f, "UDSF"),
            NfType::Bsf => write!(f, "BSF"),
            NfType::Chf => write!(f, "CHF"),
            NfType::Nwdaf => write!(f, "NWDAF"),
        }
    }
}

pub type QueryParams = HashMap<String, String>;

pub struct NrfClient {
    client: Client,
    nrf_uri: String,
    nf_instance_id: String,
    profile: Arc<RwLock<Option<NFProfile>>>,
    use_tls: bool,
}

impl NrfClient {
    pub fn new(nrf_uri: String, nf_instance_id: String, tls_config: Option<&TlsConfig>) -> Result<Self> {
        let mut use_tls = false;
        let mut client_builder = Client::builder().timeout(std::time::Duration::from_secs(30));

        if let Some(tls_cfg) = tls_config {
            if tls_cfg.enabled {
                let rustls_config = crate::tls::build_client_config(tls_cfg)
                    .context("Failed to build TLS config for NRF client")?;
                client_builder = client_builder
                    .use_preconfigured_tls(rustls_config)
                    .https_only(true);
                use_tls = true;
                info!("NRF client configured with TLS support");
            }
        }

        let client = client_builder
            .build()
            .context("Failed to build NRF HTTP client")?;

        Ok(Self {
            client,
            nrf_uri,
            nf_instance_id,
            profile: Arc::new(RwLock::new(None)),
            use_tls,
        })
    }

    pub fn build_smsf_profile(&self, smsf_host: &str, smsf_port: u16) -> NFProfile {
        let scheme = if self.use_tls { "https" } else { "http" };
        let service_base_url = format!("{}://{}:{}", scheme, smsf_host, smsf_port);

        let nf_services = vec![
            NFService {
                service_instance_id: uuid::Uuid::new_v4().to_string(),
                service_name: "nsmsf-sms".to_string(),
                versions: vec![NFServiceVersion {
                    api_version_in_uri: "v1".to_string(),
                    api_full_version: "1.0.0".to_string(),
                }],
                scheme: scheme.to_string(),
                nf_service_status: NfServiceStatus::Registered,
                ipv4_addresses: Some(vec![smsf_host.to_string()]),
                api_prefix: Some(service_base_url.clone()),
                allowed_nf_types: Some(vec![NfType::Amf]),
                priority: Some(1),
                capacity: Some(100),
                load: Some(0),
            },
        ];

        NFProfile {
            nf_instance_id: self.nf_instance_id.clone(),
            nf_type: NfType::Smsf,
            nf_status: NfStatus::Registered,
            plmn_list: vec![PlmnId {
                mcc: "001".to_string(),
                mnc: "01".to_string(),
            }],
            fqdn: None,
            ipv4_addresses: Some(vec![smsf_host.to_string()]),
            ipv6_addresses: None,
            allowed_nf_types: Some(vec![NfType::Amf]),
            priority: Some(1),
            capacity: Some(100),
            load: Some(0),
            nf_services: Some(nf_services),
            heart_beat_timer: Some(60),
        }
    }

    pub async fn register(&self, profile: NFProfile) -> Result<NFProfile> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}",
            self.nrf_uri, self.nf_instance_id
        );

        let response = self
            .client
            .put(&url)
            .json(&profile)
            .send()
            .await
            .context("Failed to send registration request to NRF")?;

        match response.status() {
            StatusCode::CREATED | StatusCode::OK => {
                let registered_profile: NFProfile = response
                    .json()
                    .await
                    .context("Failed to parse NRF registration response")?;

                let mut current_profile = self.profile.write().await;
                *current_profile = Some(registered_profile.clone());

                info!("SMSF registered with NRF successfully");
                Ok(registered_profile)
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF registration failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn deregister(&self) -> Result<()> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}",
            self.nrf_uri, self.nf_instance_id
        );

        let response = self
            .client
            .delete(&url)
            .send()
            .await
            .context("Failed to send deregistration request to NRF")?;

        match response.status() {
            StatusCode::NO_CONTENT => {
                let mut current_profile = self.profile.write().await;
                *current_profile = None;

                info!("SMSF deregistered from NRF successfully");
                Ok(())
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF deregistration failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn heartbeat(&self) -> Result<()> {
        let url = format!(
            "{}/nnrf-nfm/v1/nf-instances/{}/heartbeat",
            self.nrf_uri, self.nf_instance_id
        );

        let response = self
            .client
            .put(&url)
            .send()
            .await
            .context("Failed to send heartbeat to NRF")?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => {
                Err(anyhow::anyhow!("NF instance not found in NRF (404)"))
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF heartbeat failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn discover(
        &self,
        target_nf_type: NfType,
        query_params: Option<QueryParams>,
    ) -> Result<SearchResult> {
        let url = format!("{}/nnrf-disc/v1/nf-instances", self.nrf_uri);

        let mut params: Vec<(String, String)> = vec![
            ("target-nf-type".to_string(), target_nf_type.to_string()),
            ("requester-nf-type".to_string(), NfType::Smsf.to_string()),
        ];

        if let Some(extra) = query_params {
            params.extend(extra);
        }

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .context("Failed to send discovery request to NRF")?;

        match response.status() {
            StatusCode::OK => {
                let search_result: SearchResult = response
                    .json()
                    .await
                    .context("Failed to parse NRF discovery response")?;

                info!(
                    "Discovered {} instances of {:?} from NRF",
                    search_result.nf_instances.len(),
                    target_nf_type
                );

                Ok(search_result)
            }
            StatusCode::NOT_FOUND => Ok(SearchResult {
                validity_period: None,
                nf_instances: vec![],
                search_id: None,
                num_nf_inst_complete: Some(0),
            }),
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "NRF discovery failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }

    pub async fn start_heartbeat_task(self: Arc<Self>, smsf_host: String, smsf_port: u16) {
        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(60));
            loop {
                interval_timer.tick().await;
                match self.heartbeat().await {
                    Ok(_) => {}
                    Err(e) => {
                        if e.to_string().contains("404") {
                            info!("NRF registration lost (404), attempting re-registration");
                            let profile = self.build_smsf_profile(&smsf_host, smsf_port);
                            match self.register(profile).await {
                                Ok(_) => {
                                    info!("Successfully re-registered with NRF after 404");
                                }
                                Err(re_err) => {
                                    error!("Failed to re-register with NRF: {}", re_err);
                                }
                            }
                        } else {
                            error!("Failed to send NRF heartbeat: {}", e);
                        }
                    }
                }
            }
        });

        info!("NRF heartbeat task started");
    }
}
