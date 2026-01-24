use anyhow::{anyhow, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpMessageType {
    RpData = 0x00,
    RpAck = 0x02,
    RpError = 0x04,
    RpSmma = 0x06,
}

impl RpMessageType {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value & 0x07 {
            0x00 => Ok(RpMessageType::RpData),
            0x02 => Ok(RpMessageType::RpAck),
            0x04 => Ok(RpMessageType::RpError),
            0x06 => Ok(RpMessageType::RpSmma),
            _ => Err(anyhow!("Unknown RP message type: {:#x}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpCause {
    UnassignedNumber = 1,
    OperatorDeterminedBarring = 8,
    CallBarred = 10,
    Reserved = 11,
    ShortMessageTransferRejected = 21,
    DestinationOutOfOrder = 27,
    UnidentifiedSubscriber = 28,
    FacilityRejected = 29,
    UnknownSubscriber = 30,
    NetworkOutOfOrder = 38,
    TemporaryFailure = 41,
    Congestion = 42,
    ResourcesUnavailable = 47,
    FacilityNotSubscribed = 50,
    FacilityNotImplemented = 69,
    InvalidShortMessageTransferReference = 81,
    SemanticallyIncorrectMessage = 95,
    InvalidMandatoryInformation = 96,
    MessageTypeNonExistent = 97,
    MessageNotCompatible = 98,
    InformationElementNonExistent = 99,
    ProtocolError = 111,
    Interworking = 127,
}

impl RpCause {
    pub fn from_u8(value: u8) -> Result<Self> {
        match value {
            1 => Ok(RpCause::UnassignedNumber),
            8 => Ok(RpCause::OperatorDeterminedBarring),
            10 => Ok(RpCause::CallBarred),
            11 => Ok(RpCause::Reserved),
            21 => Ok(RpCause::ShortMessageTransferRejected),
            27 => Ok(RpCause::DestinationOutOfOrder),
            28 => Ok(RpCause::UnidentifiedSubscriber),
            29 => Ok(RpCause::FacilityRejected),
            30 => Ok(RpCause::UnknownSubscriber),
            38 => Ok(RpCause::NetworkOutOfOrder),
            41 => Ok(RpCause::TemporaryFailure),
            42 => Ok(RpCause::Congestion),
            47 => Ok(RpCause::ResourcesUnavailable),
            50 => Ok(RpCause::FacilityNotSubscribed),
            69 => Ok(RpCause::FacilityNotImplemented),
            81 => Ok(RpCause::InvalidShortMessageTransferReference),
            95 => Ok(RpCause::SemanticallyIncorrectMessage),
            96 => Ok(RpCause::InvalidMandatoryInformation),
            97 => Ok(RpCause::MessageTypeNonExistent),
            98 => Ok(RpCause::MessageNotCompatible),
            99 => Ok(RpCause::InformationElementNonExistent),
            111 => Ok(RpCause::ProtocolError),
            127 => Ok(RpCause::Interworking),
            _ => Ok(RpCause::ProtocolError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpDirection {
    MobileOriginated,
    MobileTerminated,
}

#[derive(Debug, Clone)]
pub enum RpMessage {
    RpData {
        message_reference: u8,
        originator_or_destination: Option<String>,
        user_data: Vec<u8>,
        direction: RpDirection,
    },
    RpAck {
        message_reference: u8,
        user_data: Option<Vec<u8>>,
    },
    RpError {
        message_reference: u8,
        cause: RpCause,
        diagnostic: Option<u8>,
    },
    RpSmma {
        message_reference: u8,
    },
}

fn encode_address(addr: &str) -> Vec<u8> {
    let mut encoded = Vec::new();

    let digits: Vec<char> = addr.chars().filter(|c| c.is_ascii_digit()).collect();

    encoded.push(digits.len() as u8);

    let ton_npi = if addr.starts_with('+') {
        0x91
    } else {
        0x81
    };
    encoded.push(ton_npi);

    let mut i = 0;
    while i < digits.len() {
        let low = digits[i].to_digit(10).unwrap() as u8;
        let high = if i + 1 < digits.len() {
            digits[i + 1].to_digit(10).unwrap() as u8
        } else {
            0x0F
        };
        encoded.push((high << 4) | low);
        i += 2;
    }

    encoded
}

fn decode_address(data: &[u8]) -> Result<(Option<String>, usize)> {
    if data.is_empty() {
        return Ok((None, 0));
    }

    let len = data[0] as usize;
    if len == 0 {
        return Ok((None, 1));
    }

    if data.len() < 2 {
        return Err(anyhow!("Address data too short"));
    }

    let ton_npi = data[1];
    let num_octets = (len + 1) / 2;

    if data.len() < 2 + num_octets {
        return Err(anyhow!("Address data incomplete"));
    }

    let mut digits = String::new();
    for i in 0..num_octets {
        let octet = data[2 + i];
        let low = octet & 0x0F;
        let high = (octet >> 4) & 0x0F;

        digits.push(char::from_digit(low as u32, 10).unwrap_or('0'));
        if high != 0x0F {
            digits.push(char::from_digit(high as u32, 10).unwrap_or('0'));
        }
    }

    let addr = if (ton_npi & 0x70) == 0x10 {
        format!("+{}", digits)
    } else {
        digits
    };

    Ok((Some(addr), 2 + num_octets))
}

impl RpMessage {
    pub fn encode(&self) -> Vec<u8> {
        let mut msg = Vec::new();

        match self {
            RpMessage::RpData {
                message_reference,
                originator_or_destination,
                user_data,
                direction,
            } => {
                let mti = match direction {
                    RpDirection::MobileOriginated => 0x00,
                    RpDirection::MobileTerminated => 0x01,
                };
                msg.push(mti);
                msg.push(*message_reference);

                if let Some(addr) = originator_or_destination {
                    let encoded_addr = encode_address(addr);
                    msg.extend_from_slice(&encoded_addr);
                } else {
                    msg.push(0);
                }

                msg.push(user_data.len() as u8);
                msg.extend_from_slice(user_data);
            }
            RpMessage::RpAck {
                message_reference,
                user_data,
            } => {
                msg.push(RpMessageType::RpAck as u8);
                msg.push(*message_reference);

                if let Some(ud) = user_data {
                    msg.push(ud.len() as u8);
                    msg.extend_from_slice(ud);
                }
            }
            RpMessage::RpError {
                message_reference,
                cause,
                diagnostic,
            } => {
                msg.push(RpMessageType::RpError as u8);
                msg.push(*message_reference);

                msg.push(*cause as u8);

                if let Some(diag) = diagnostic {
                    msg.push(*diag);
                }
            }
            RpMessage::RpSmma { message_reference } => {
                msg.push(RpMessageType::RpSmma as u8);
                msg.push(*message_reference);
            }
        }

        msg
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 2 {
            return Err(anyhow!("RP message too short"));
        }

        let msg_type = RpMessageType::from_u8(data[0])?;
        let message_reference = data[1];

        match msg_type {
            RpMessageType::RpData => {
                let direction = if data[0] & 0x01 == 0 {
                    RpDirection::MobileOriginated
                } else {
                    RpDirection::MobileTerminated
                };

                let (addr, addr_len) = decode_address(&data[2..])?;
                let pos = 2 + addr_len;

                if data.len() < pos + 1 {
                    return Err(anyhow!("RP-DATA too short for user data length"));
                }

                let ud_len = data[pos] as usize;
                if data.len() < pos + 1 + ud_len {
                    return Err(anyhow!("RP-DATA user data incomplete"));
                }

                let user_data = data[pos + 1..pos + 1 + ud_len].to_vec();

                Ok(RpMessage::RpData {
                    message_reference,
                    originator_or_destination: addr,
                    user_data,
                    direction,
                })
            }
            RpMessageType::RpAck => {
                let user_data = if data.len() > 2 {
                    let ud_len = data[2] as usize;
                    if data.len() >= 3 + ud_len {
                        Some(data[3..3 + ud_len].to_vec())
                    } else {
                        None
                    }
                } else {
                    None
                };

                Ok(RpMessage::RpAck {
                    message_reference,
                    user_data,
                })
            }
            RpMessageType::RpError => {
                if data.len() < 3 {
                    return Err(anyhow!("RP-ERROR too short"));
                }

                let cause = RpCause::from_u8(data[2])?;
                let diagnostic = if data.len() > 3 { Some(data[3]) } else { None };

                Ok(RpMessage::RpError {
                    message_reference,
                    cause,
                    diagnostic,
                })
            }
            RpMessageType::RpSmma => Ok(RpMessage::RpSmma { message_reference }),
        }
    }

    pub fn message_reference(&self) -> u8 {
        match self {
            RpMessage::RpData {
                message_reference, ..
            } => *message_reference,
            RpMessage::RpAck {
                message_reference, ..
            } => *message_reference,
            RpMessage::RpError {
                message_reference, ..
            } => *message_reference,
            RpMessage::RpSmma { message_reference } => *message_reference,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rp_ack_encode_decode() {
        let rp_ack = RpMessage::RpAck {
            message_reference: 42,
            user_data: None,
        };
        let encoded = rp_ack.encode();
        assert_eq!(encoded, vec![0x02, 42]);

        let decoded = RpMessage::decode(&encoded).unwrap();
        match decoded {
            RpMessage::RpAck {
                message_reference, ..
            } => assert_eq!(message_reference, 42),
            _ => panic!("Expected RpAck"),
        }
    }

    #[test]
    fn test_rp_error_encode_decode() {
        let rp_error = RpMessage::RpError {
            message_reference: 10,
            cause: RpCause::Congestion,
            diagnostic: Some(0x01),
        };
        let encoded = rp_error.encode();

        let decoded = RpMessage::decode(&encoded).unwrap();
        match decoded {
            RpMessage::RpError {
                message_reference,
                cause,
                diagnostic,
            } => {
                assert_eq!(message_reference, 10);
                assert_eq!(cause, RpCause::Congestion);
                assert_eq!(diagnostic, Some(0x01));
            }
            _ => panic!("Expected RpError"),
        }
    }

    #[test]
    fn test_rp_smma_encode_decode() {
        let rp_smma = RpMessage::RpSmma {
            message_reference: 99,
        };
        let encoded = rp_smma.encode();
        assert_eq!(encoded, vec![0x06, 99]);

        let decoded = RpMessage::decode(&encoded).unwrap();
        match decoded {
            RpMessage::RpSmma { message_reference } => assert_eq!(message_reference, 99),
            _ => panic!("Expected RpSmma"),
        }
    }

    #[test]
    fn test_rp_data_mo_encode_decode() {
        let user_data = vec![0x11, 0x22, 0x33];
        let rp_data = RpMessage::RpData {
            message_reference: 5,
            originator_or_destination: Some("+1234567890".to_string()),
            user_data: user_data.clone(),
            direction: RpDirection::MobileOriginated,
        };

        let encoded = rp_data.encode();
        let decoded = RpMessage::decode(&encoded).unwrap();

        match decoded {
            RpMessage::RpData {
                message_reference,
                originator_or_destination,
                user_data: decoded_ud,
                direction,
            } => {
                assert_eq!(message_reference, 5);
                assert_eq!(
                    originator_or_destination,
                    Some("+1234567890".to_string())
                );
                assert_eq!(decoded_ud, user_data);
                assert_eq!(direction, RpDirection::MobileOriginated);
            }
            _ => panic!("Expected RpData"),
        }
    }
}
