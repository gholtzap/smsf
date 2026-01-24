use anyhow::{Context, Result};
use base64::Engine;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::info;

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

#[derive(Clone)]
pub struct AmfClient {
    client: Client,
}

impl AmfClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
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
