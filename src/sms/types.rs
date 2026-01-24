use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SmsDeliveryStatus {
    Pending,
    Accepted,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRecord {
    pub sms_record_id: String,
    pub sms_payload: Vec<u8>,
    pub delivery_status: SmsDeliveryStatus,
    pub gpsi: Option<String>,
    pub supi: String,
    pub amf_id: String,
    pub retry_count: u32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status_report_requested: bool,
    pub originator_address: Option<String>,
    pub message_reference: Option<u8>,
    pub is_mobile_originated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsDeliveryData {
    pub sms_record_id: String,
    pub sms_msg: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccessType {
    #[serde(rename = "3GPP_ACCESS")]
    ThreeGppAccess,
    #[serde(rename = "NON_3GPP_ACCESS")]
    Non3GppAccess,
}
