use anyhow::{Context, Result};
use base64::Engine;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::sync::Arc;
use tracing::{info, warn};

use super::nrf::{NfServiceStatus, NfType, NrfClient};
use crate::config::TlsConfig;
use crate::sbi::models::{Guami, UserLocation};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N1N2MessageTransferRequest {
    pub n1_message_container: Option<N1MessageContainer>,
    pub n2_info_container: Option<N2InfoContainer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N1MessageContainer {
    pub n1_message_class: String,
    pub n1_message_content: N1MessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N1MessageContent {
    pub content_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N2InfoContainer {
    pub n2_information_class: String,
    pub n2_sm_info: N2SmInformation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct N2SmInformation {
    pub content_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UeContextInfo {
    pub supi: String,
    #[serde(rename = "cmState")]
    pub cm_state: CmState,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CmState {
    Connected,
    Idle,
}

#[derive(Clone)]
pub struct AmfClient {
    client: Client,
    nrf_client: Option<Arc<NrfClient>>,
}

impl AmfClient {
    pub fn new(tls_config: Option<&TlsConfig>) -> Result<Self> {
        let client = Self::build_client(tls_config)?;
        Ok(Self {
            client,
            nrf_client: None,
        })
    }

    pub fn with_nrf(nrf_client: Arc<NrfClient>, tls_config: Option<&TlsConfig>) -> Result<Self> {
        let client = Self::build_client(tls_config)?;
        Ok(Self {
            client,
            nrf_client: Some(nrf_client),
        })
    }

    fn build_client(tls_config: Option<&TlsConfig>) -> Result<Client> {
        let mut client_builder = Client::builder().timeout(std::time::Duration::from_secs(30));

        if let Some(tls_cfg) = tls_config {
            if tls_cfg.enabled {
                let rustls_config = crate::tls::build_client_config(tls_cfg)
                    .context("Failed to build TLS config for AMF client")?;
                client_builder = client_builder
                    .use_preconfigured_tls(rustls_config)
                    .https_only(true);
                info!("AMF client configured with TLS support");
            }
        }

        client_builder
            .build()
            .context("Failed to build AMF HTTP client")
    }

    pub async fn discover_amf(
        &self,
        guami: Option<&Guami>,
        _ue_location: Option<&UserLocation>,
    ) -> Result<String> {
        let nrf_client = self
            .nrf_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NRF client not configured"))?;

        let mut query_params = std::collections::HashMap::new();

        if let Some(guami) = guami {
            let guami_str = serde_json::to_string(&guami)
                .context("Failed to serialize GUAMI for NRF discovery")?;
            query_params.insert("guami".to_string(), guami_str);
        }

        let search_result = nrf_client
            .discover(NfType::Amf, Some(query_params))
            .await
            .context("Failed to discover AMF from NRF")?;

        if search_result.nf_instances.is_empty() {
            return Err(anyhow::anyhow!("No AMF instances found"));
        }

        let amf_instance = search_result
            .nf_instances
            .iter()
            .min_by_key(|inst| {
                (
                    inst.priority.unwrap_or(u16::MAX),
                    Reverse(inst.capacity.unwrap_or(0)),
                    inst.load.unwrap_or(u16::MAX),
                )
            })
            .ok_or_else(|| anyhow::anyhow!("No valid AMF instance found"))?;

        if let Some(ref services) = amf_instance.nf_services {
            for service in services {
                if service.service_name == "namf-comm"
                    && service.nf_service_status == NfServiceStatus::Registered
                {
                    if let Some(ref api_prefix) = service.api_prefix {
                        info!("Discovered AMF at {}", api_prefix);
                        return Ok(api_prefix.clone());
                    }
                }
            }
        }

        if let Some(ref ipv4_addrs) = amf_instance.ipv4_addresses {
            if let Some(first_addr) = ipv4_addrs.first() {
                let amf_uri = format!("http://{}", first_addr);
                info!("Discovered AMF at {}", amf_uri);
                return Ok(amf_uri);
            }
        }

        Err(anyhow::anyhow!("No valid AMF endpoint found"))
    }

    pub async fn check_ue_reachability(&self, supi: &str, amf_uri: &str) -> Result<bool> {
        let url = format!("{}/namf-comm/v1/ue-contexts/{}", amf_uri, supi);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to check UE context at AMF")?;

        match response.status() {
            StatusCode::OK => {
                let context_info: UeContextInfo = response
                    .json()
                    .await
                    .context("Failed to parse UE context response")?;

                let is_reachable = context_info.cm_state == CmState::Connected;
                info!(
                    "UE {} reachability: {} (CM state: {:?})",
                    supi, is_reachable, context_info.cm_state
                );
                Ok(is_reachable)
            }
            StatusCode::NOT_FOUND => {
                warn!("UE context not found at AMF for SUPI: {}", supi);
                Ok(false)
            }
            status => {
                warn!(
                    "Failed to check UE reachability at AMF, status: {}",
                    status
                );
                Ok(false)
            }
        }
    }

    pub async fn send_n1n2_message(
        &self,
        supi: &str,
        amf_uri: &str,
        sms_payload: Vec<u8>,
    ) -> Result<()> {
        let url = format!(
            "{}/namf-comm/v1/ue-contexts/{}/n1-n2-messages",
            amf_uri, supi
        );

        let sms_payload_b64 = base64::engine::general_purpose::STANDARD.encode(&sms_payload);

        let boundary = "----SMSFMessageBoundary";
        let body = format!(
            "--{}\r\nContent-Type: application/json\r\n\r\n{}\r\n--{}\r\nContent-Type: application/vnd.3gpp.sms\r\nContent-Id: sms-payload\r\n\r\n{}\r\n--{}--\r\n",
            boundary,
            serde_json::to_string(&serde_json::json!({
                "n1MessageContainer": {
                    "n1MessageClass": "SMS",
                    "n1MessageContent": {
                        "contentId": "sms-payload"
                    }
                }
            }))?,
            boundary,
            sms_payload_b64,
            boundary
        );

        let response = self
            .client
            .post(&url)
            .header(
                "Content-Type",
                format!("multipart/related; boundary={}", boundary),
            )
            .body(body)
            .send()
            .await
            .context("Failed to send N1N2 message to AMF")?;

        match response.status() {
            StatusCode::OK | StatusCode::ACCEPTED => {
                info!("N1N2 message sent to AMF successfully");
                Ok(())
            }
            status => {
                let error_body = response.text().await.unwrap_or_default();
                Err(anyhow::anyhow!(
                    "N1N2 message transfer failed with status {}: {}",
                    status,
                    error_body
                ))
            }
        }
    }
}
