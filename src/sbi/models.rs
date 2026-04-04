use crate::sms::types::{AccessType, SmsDeliveryStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guami {
    #[serde(rename = "plmnId")]
    pub plmn_id: PlmnId,
    #[serde(rename = "amfId")]
    pub amf_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlmnId {
    pub mcc: String,
    pub mnc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tai {
    #[serde(rename = "plmnId")]
    pub plmn_id: PlmnId,
    pub tac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    #[serde(rename = "nrLocation", skip_serializing_if = "Option::is_none")]
    pub nr_location: Option<NrLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NrLocation {
    pub tai: Tai,
    pub ncgi: Ncgi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ncgi {
    #[serde(rename = "plmnId")]
    pub plmn_id: PlmnId,
    #[serde(rename = "nrCellId")]
    pub nr_cell_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UeSmsContextData {
    pub supi: String,
    #[serde(rename = "amfId")]
    pub amf_id: String,
    #[serde(rename = "accessType")]
    pub access_type: AccessType,
    pub guami: Option<Guami>,
    #[serde(rename = "ueLocation", skip_serializing_if = "Option::is_none")]
    pub ue_location: Option<UserLocation>,
    pub gpsi: Option<String>,
    #[serde(rename = "ueTimeZone", skip_serializing_if = "Option::is_none")]
    pub ue_time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefToBinaryData {
    #[serde(rename = "contentId")]
    pub content_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRecordData {
    #[serde(rename = "smsRecordId")]
    pub sms_record_id: String,
    #[serde(rename = "smsPayload")]
    pub sms_payload: RefToBinaryData,
    #[serde(rename = "accessType", skip_serializing_if = "Option::is_none")]
    pub access_type: Option<AccessType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpsi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pei: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmsDeliveryReportStatus {
    #[serde(rename = "SMS_DELIVERY_PENDING")]
    Pending,
    #[serde(rename = "SMS_DELIVERY_COMPLETED")]
    Completed,
    #[serde(rename = "SMS_DELIVERY_FAILED")]
    Failed,
    #[serde(rename = "SMS_DELIVERY_SMSF_ACCEPTED")]
    SmsfAccepted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsRecordDeliveryData {
    #[serde(rename = "smsRecordId")]
    pub sms_record_id: String,
    #[serde(rename = "deliveryStatus")]
    pub delivery_status: SmsDeliveryReportStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsDeliveryReportData {
    #[serde(rename = "smsRecordId")]
    pub sms_record_id: String,
    #[serde(rename = "deliveryStatus")]
    pub delivery_status: SmsDeliveryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemDetails {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub problem_type: Option<String>,
    pub title: Option<String>,
    pub status: u16,
    pub detail: Option<String>,
    pub instance: Option<String>,
}

impl ProblemDetails {
    pub fn new(status: u16, detail: String) -> Self {
        Self {
            problem_type: None,
            title: None,
            status,
            detail: Some(detail),
            instance: None,
        }
    }

    pub fn not_found(detail: String) -> Self {
        Self::new(404, detail)
    }

    pub fn bad_request(detail: String) -> Self {
        Self::new(400, detail)
    }

    pub fn internal_error(detail: String) -> Self {
        Self::new(500, detail)
    }

}
