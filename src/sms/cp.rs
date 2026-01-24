use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpMessageType {
    CpData = 0x01,
    CpAck = 0x04,
    CpError = 0x10,
}

impl CpMessageType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            0x01 => Ok(CpMessageType::CpData),
            0x04 => Ok(CpMessageType::CpAck),
            0x10 => Ok(CpMessageType::CpError),
            _ => Err(anyhow!("Unknown CP message type: {:#x}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpCause {
    NetworkFailure = 17,
    CongestionProtocolError = 42,
    InformationElementNonExistent = 99,
    MessageTypeNonExistent = 97,
    MessageNotCompatible = 98,
    ConditionalIeError = 100,
    Unspecified = 111,
}

impl CpCause {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            17 => Ok(CpCause::NetworkFailure),
            42 => Ok(CpCause::CongestionProtocolError),
            97 => Ok(CpCause::MessageTypeNonExistent),
            98 => Ok(CpCause::MessageNotCompatible),
            99 => Ok(CpCause::InformationElementNonExistent),
            100 => Ok(CpCause::ConditionalIeError),
            111 => Ok(CpCause::Unspecified),
            _ => Ok(CpCause::Unspecified),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CpMessage {
    CpData { rp_data: Vec<u8> },
    CpAck,
    CpError { cause: CpCause },
}

impl CpMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut msg = Vec::new();

        match self {
            CpMessage::CpData { rp_data } => {
                msg.push(CpMessageType::CpData as u8);
                msg.push(rp_data.len() as u8);
                msg.extend_from_slice(rp_data);
            }
            CpMessage::CpAck => {
                msg.push(CpMessageType::CpAck as u8);
            }
            CpMessage::CpError { cause } => {
                msg.push(CpMessageType::CpError as u8);
                msg.push(*cause as u8);
            }
        }

        msg
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.is_empty() {
            return Err(anyhow!("CP message is empty"));
        }

        let msg_type = CpMessageType::from_u8(data[0])?;

        match msg_type {
            CpMessageType::CpData => {
                if data.len() < 2 {
                    return Err(anyhow!("CP-DATA too short"));
                }
                let rp_data_len = data[1] as usize;
                if data.len() < 2 + rp_data_len {
                    return Err(anyhow!("CP-DATA length mismatch"));
                }
                let rp_data = data[2..2 + rp_data_len].to_vec();
                Ok(CpMessage::CpData { rp_data })
            }
            CpMessageType::CpAck => Ok(CpMessage::CpAck),
            CpMessageType::CpError => {
                if data.len() < 2 {
                    return Err(anyhow!("CP-ERROR too short"));
                }
                let cause = CpCause::from_u8(data[1])?;
                Ok(CpMessage::CpError { cause })
            }
        }
    }

    pub fn message_type(&self) -> CpMessageType {
        match self {
            CpMessage::CpData { .. } => CpMessageType::CpData,
            CpMessage::CpAck => CpMessageType::CpAck,
            CpMessage::CpError { .. } => CpMessageType::CpError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cp_ack_encode_decode() {
        let cp_ack = CpMessage::CpAck;
        let encoded = cp_ack.encode();
        assert_eq!(encoded, vec![0x04]);

        let decoded = CpMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.message_type(), CpMessageType::CpAck);
    }

    #[test]
    fn test_cp_error_encode_decode() {
        let cp_error = CpMessage::CpError {
            cause: CpCause::NetworkFailure,
        };
        let encoded = cp_error.encode();
        assert_eq!(encoded, vec![0x10, 17]);

        let decoded = CpMessage::decode(&encoded).unwrap();
        match decoded {
            CpMessage::CpError { cause } => assert_eq!(cause, CpCause::NetworkFailure),
            _ => panic!("Expected CpError"),
        }
    }

    #[test]
    fn test_cp_data_encode_decode() {
        let rp_data = vec![0x01, 0x02, 0x03, 0x04];
        let cp_data = CpMessage::CpData {
            rp_data: rp_data.clone(),
        };
        let encoded = cp_data.encode();

        assert_eq!(encoded[0], 0x01);
        assert_eq!(encoded[1], 4);
        assert_eq!(&encoded[2..], &rp_data[..]);

        let decoded = CpMessage::decode(&encoded).unwrap();
        match decoded {
            CpMessage::CpData { rp_data: decoded_rp } => assert_eq!(decoded_rp, rp_data),
            _ => panic!("Expected CpData"),
        }
    }
}
