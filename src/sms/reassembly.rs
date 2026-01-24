use super::encoding::{decode_text, DataCodingScheme};
use super::tpdu::{TpDeliver, TpSubmit};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessagePart {
    pub sequence: u8,
    pub data: Vec<u8>,
    pub dcs: DataCodingScheme,
    pub user_data_length: u8,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct PendingMessage {
    pub reference: u16,
    pub total_parts: u8,
    pub parts: HashMap<u8, MessagePart>,
    pub first_received: DateTime<Utc>,
    pub last_received: DateTime<Utc>,
    pub originator: String,
}

pub struct MessageReassembly {
    pending: Arc<Mutex<HashMap<String, PendingMessage>>>,
    timeout: Duration,
}

impl MessageReassembly {
    pub fn new(timeout_minutes: i64) -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
            timeout: Duration::minutes(timeout_minutes),
        }
    }

    pub async fn add_part_submit(
        &self,
        submit: &TpSubmit,
    ) -> Result<Option<String>> {
        let concat_info = submit
            .udh
            .as_ref()
            .and_then(|u| u.get_concat_info())
            .ok_or_else(|| anyhow!("Not a concatenated message"))?;

        let key = format!(
            "{}:{}",
            submit.destination_address, concat_info.reference
        );

        let mut pending = self.pending.lock().await;

        let pending_msg = pending
            .entry(key.clone())
            .or_insert_with(|| PendingMessage {
                reference: concat_info.reference,
                total_parts: concat_info.total_parts,
                parts: HashMap::new(),
                first_received: Utc::now(),
                last_received: Utc::now(),
                originator: submit.destination_address.clone(),
            });

        pending_msg.last_received = Utc::now();

        let part = MessagePart {
            sequence: concat_info.sequence,
            data: submit.user_data.clone(),
            dcs: submit.data_coding_scheme,
            user_data_length: submit.user_data_length,
            received_at: Utc::now(),
        };

        pending_msg.parts.insert(concat_info.sequence, part);

        if pending_msg.parts.len() == pending_msg.total_parts as usize {
            let complete_msg = self.reassemble_message(pending_msg)?;
            pending.remove(&key);
            Ok(Some(complete_msg))
        } else {
            Ok(None)
        }
    }

    pub async fn add_part_deliver(
        &self,
        deliver: &TpDeliver,
    ) -> Result<Option<String>> {
        let concat_info = deliver
            .udh
            .as_ref()
            .and_then(|u| u.get_concat_info())
            .ok_or_else(|| anyhow!("Not a concatenated message"))?;

        let key = format!(
            "{}:{}",
            deliver.originating_address, concat_info.reference
        );

        let mut pending = self.pending.lock().await;

        let pending_msg = pending
            .entry(key.clone())
            .or_insert_with(|| PendingMessage {
                reference: concat_info.reference,
                total_parts: concat_info.total_parts,
                parts: HashMap::new(),
                first_received: Utc::now(),
                last_received: Utc::now(),
                originator: deliver.originating_address.clone(),
            });

        pending_msg.last_received = Utc::now();

        let part = MessagePart {
            sequence: concat_info.sequence,
            data: deliver.user_data.clone(),
            dcs: deliver.data_coding_scheme,
            user_data_length: deliver.user_data_length,
            received_at: Utc::now(),
        };

        pending_msg.parts.insert(concat_info.sequence, part);

        if pending_msg.parts.len() == pending_msg.total_parts as usize {
            let complete_msg = self.reassemble_message(pending_msg)?;
            pending.remove(&key);
            Ok(Some(complete_msg))
        } else {
            Ok(None)
        }
    }

    fn reassemble_message(&self, pending_msg: &PendingMessage) -> Result<String> {
        let mut sorted_parts: Vec<_> = pending_msg.parts.iter().collect();
        sorted_parts.sort_by_key(|(seq, _)| *seq);

        if sorted_parts.is_empty() {
            return Err(anyhow!("No parts to reassemble"));
        }

        let first_part = sorted_parts[0].1;
        let dcs = first_part.dcs;

        let mut combined_data = Vec::new();
        let mut total_length = 0;

        for (_, part) in sorted_parts {
            if part.dcs != dcs {
                return Err(anyhow!("Inconsistent data coding scheme across parts"));
            }
            combined_data.extend_from_slice(&part.data);
            total_length += part.user_data_length as usize;
        }

        decode_text(&combined_data, dcs, total_length)
    }

    pub async fn cleanup_expired(&self) {
        let mut pending = self.pending.lock().await;
        let now = Utc::now();

        pending.retain(|_, msg| {
            now.signed_duration_since(msg.last_received) < self.timeout
        });
    }

    pub async fn get_pending_count(&self) -> usize {
        self.pending.lock().await.len()
    }

    pub async fn get_pending_info(&self, originator: &str) -> Vec<(u16, u8, u8)> {
        let pending = self.pending.lock().await;
        pending
            .iter()
            .filter(|(_, msg)| msg.originator == originator)
            .map(|(_, msg)| (msg.reference, msg.total_parts, msg.parts.len() as u8))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sms::concatenation::split_long_message_deliver;

    #[tokio::test]
    async fn test_reassembly_deliver() {
        let reassembly = MessageReassembly::new(10);
        let text = "A".repeat(200);
        let orig = "+1234567890".to_string();

        let parts = split_long_message_deliver(orig.clone(), text.clone()).unwrap();
        assert!(parts.len() > 1);

        let mut result = None;
        for part in &parts {
            result = reassembly.add_part_deliver(part).await.unwrap();
        }

        assert!(result.is_some());
        assert_eq!(result.unwrap(), text);
    }

    #[tokio::test]
    async fn test_partial_message() {
        let reassembly = MessageReassembly::new(10);
        let text = "B".repeat(200);
        let orig = "+1234567890".to_string();

        let parts = split_long_message_deliver(orig.clone(), text.clone()).unwrap();

        let result = reassembly.add_part_deliver(&parts[0]).await.unwrap();
        assert!(result.is_none());

        assert_eq!(reassembly.get_pending_count().await, 1);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let reassembly = MessageReassembly::new(0);
        let text = "C".repeat(200);
        let orig = "+1234567890".to_string();

        let parts = split_long_message_deliver(orig.clone(), text.clone()).unwrap();
        reassembly.add_part_deliver(&parts[0]).await.unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        reassembly.cleanup_expired().await;

        assert_eq!(reassembly.get_pending_count().await, 0);
    }
}
