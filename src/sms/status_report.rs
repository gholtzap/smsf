use super::tpdu::{TpDeliver, TpStatusReport};
use super::types::{SmsDeliveryData, SmsDeliveryStatus, SmsRecord};
use crate::db::Database;
use crate::sms::delivery::SmsDeliveryService;
use anyhow::Result;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct StatusReportService {
    db: Database,
    delivery_service: Arc<SmsDeliveryService>,
}

impl StatusReportService {
    pub fn new(db: Database, delivery_service: Arc<SmsDeliveryService>) -> Self {
        Self {
            db,
            delivery_service,
        }
    }

    pub async fn send_status_report(&self, sms_record: &SmsRecord) -> Result<()> {
        if !sms_record.status_report_requested {
            return Ok(());
        }

        let originator = match &sms_record.originator_address {
            Some(addr) => addr,
            None => {
                warn!(
                    "Cannot send status report: no originator address for SMS record {}",
                    sms_record.sms_record_id
                );
                return Ok(());
            }
        };

        let message_reference = match sms_record.message_reference {
            Some(mr) => mr,
            None => {
                warn!(
                    "Cannot send status report: no message reference for SMS record {}",
                    sms_record.sms_record_id
                );
                return Ok(());
            }
        };

        let status = match sms_record.delivery_status {
            SmsDeliveryStatus::Completed => 0x00,
            SmsDeliveryStatus::Failed => 0x41,
            SmsDeliveryStatus::Pending => 0x20,
            SmsDeliveryStatus::Accepted => 0x00,
        };

        let recipient = match &sms_record.gpsi {
            Some(gpsi) => gpsi.clone(),
            None => sms_record.supi.clone(),
        };

        let status_report = TpStatusReport::new(message_reference, recipient, status);

        let status_report_pdu = status_report.encode();
        let pdu_len = status_report_pdu.len() as u8;

        let deliver_wrapper = TpDeliver {
            originating_address: originator.clone(),
            protocol_identifier: 0,
            data_coding_scheme: super::encoding::DataCodingScheme::Gsm7Bit,
            timestamp: status_report.timestamp,
            user_data: status_report_pdu,
            user_data_length: pdu_len,
            more_messages_to_send: false,
            status_report_indication: true,
            reply_path: false,
            udh: None,
        };

        let deliver_pdu = deliver_wrapper.encode();

        let sms_data = SmsDeliveryData {
            sms_record_id: format!("status-report-{}", sms_record.sms_record_id),
            sms_msg: deliver_pdu,
        };

        match self
            .delivery_service
            .deliver_mt_sms(&sms_record.supi, sms_data)
            .await
        {
            Ok(_) => {
                info!(
                    "Status report sent for SMS record {}",
                    sms_record.sms_record_id
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to send status report for SMS record {}: {}",
                    sms_record.sms_record_id, e
                );
                Err(e)
            }
        }
    }

    pub async fn handle_delivery_status_change(
        &self,
        sms_record_id: &str,
        new_status: SmsDeliveryStatus,
    ) -> Result<()> {
        if !matches!(
            new_status,
            SmsDeliveryStatus::Completed | SmsDeliveryStatus::Failed
        ) {
            return Ok(());
        }

        let sms_record = match self.db.get_sms_record(sms_record_id).await? {
            Some(record) => record,
            None => {
                warn!("SMS record not found: {}", sms_record_id);
                return Ok(());
            }
        };

        if !sms_record.is_mobile_originated {
            return Ok(());
        }

        self.send_status_report(&sms_record).await
    }
}
