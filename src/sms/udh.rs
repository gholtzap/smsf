use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum InformationElement {
    Concatenated8Bit {
        reference: u8,
        total_parts: u8,
        sequence: u8,
    },
    Concatenated16Bit {
        reference: u16,
        total_parts: u8,
        sequence: u8,
    },
    ApplicationPort8Bit {
        destination_port: u8,
        source_port: u8,
    },
    ApplicationPort16Bit {
        destination_port: u16,
        source_port: u16,
    },
    Unknown {
        iei: u8,
        data: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct UserDataHeader {
    pub elements: Vec<InformationElement>,
    pub total_length: usize,
}

impl UserDataHeader {
    pub fn new(elements: Vec<InformationElement>) -> Self {
        let total_length = 1 + elements.iter().map(|e| e.encoded_length()).sum::<usize>();
        Self {
            elements,
            total_length,
        }
    }

    pub fn parse(data: &[u8]) -> Result<(Self, usize)> {
        if data.is_empty() {
            return Err(anyhow!("Empty UDH data"));
        }

        let udhl = data[0] as usize;
        if data.len() < udhl + 1 {
            return Err(anyhow!("UDH data too short"));
        }

        let mut elements = Vec::new();
        let mut pos = 1;

        while pos < udhl + 1 {
            if pos + 1 >= data.len() {
                break;
            }

            let iei = data[pos];
            let iedl = data[pos + 1] as usize;
            pos += 2;

            if pos + iedl > data.len() {
                return Err(anyhow!("Invalid IE length"));
            }

            let ie_data = &data[pos..pos + iedl];
            let element = match iei {
                0x00 => {
                    if iedl == 3 {
                        InformationElement::Concatenated8Bit {
                            reference: ie_data[0],
                            total_parts: ie_data[1],
                            sequence: ie_data[2],
                        }
                    } else {
                        InformationElement::Unknown {
                            iei,
                            data: ie_data.to_vec(),
                        }
                    }
                }
                0x08 => {
                    if iedl == 4 {
                        InformationElement::Concatenated16Bit {
                            reference: u16::from_be_bytes([ie_data[0], ie_data[1]]),
                            total_parts: ie_data[2],
                            sequence: ie_data[3],
                        }
                    } else {
                        InformationElement::Unknown {
                            iei,
                            data: ie_data.to_vec(),
                        }
                    }
                }
                0x04 => {
                    if iedl == 2 {
                        InformationElement::ApplicationPort8Bit {
                            destination_port: ie_data[0],
                            source_port: ie_data[1],
                        }
                    } else {
                        InformationElement::Unknown {
                            iei,
                            data: ie_data.to_vec(),
                        }
                    }
                }
                0x05 => {
                    if iedl == 4 {
                        InformationElement::ApplicationPort16Bit {
                            destination_port: u16::from_be_bytes([ie_data[0], ie_data[1]]),
                            source_port: u16::from_be_bytes([ie_data[2], ie_data[3]]),
                        }
                    } else {
                        InformationElement::Unknown {
                            iei,
                            data: ie_data.to_vec(),
                        }
                    }
                }
                _ => InformationElement::Unknown {
                    iei,
                    data: ie_data.to_vec(),
                },
            };

            elements.push(element);
            pos += iedl;
        }

        Ok((
            UserDataHeader {
                elements,
                total_length: udhl + 1,
            },
            udhl + 1,
        ))
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();

        let udhl = self.elements.iter().map(|e| e.encoded_length()).sum::<usize>();
        result.push(udhl as u8);

        for element in &self.elements {
            result.extend_from_slice(&element.encode());
        }

        result
    }

    pub fn get_concat_info(&self) -> Option<ConcatenationInfo> {
        for element in &self.elements {
            match element {
                InformationElement::Concatenated8Bit {
                    reference,
                    total_parts,
                    sequence,
                } => {
                    return Some(ConcatenationInfo {
                        reference: *reference as u16,
                        total_parts: *total_parts,
                        sequence: *sequence,
                    });
                }
                InformationElement::Concatenated16Bit {
                    reference,
                    total_parts,
                    sequence,
                } => {
                    return Some(ConcatenationInfo {
                        reference: *reference,
                        total_parts: *total_parts,
                        sequence: *sequence,
                    });
                }
                _ => {}
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConcatenationInfo {
    pub reference: u16,
    pub total_parts: u8,
    pub sequence: u8,
}

impl InformationElement {
    fn encoded_length(&self) -> usize {
        match self {
            InformationElement::Concatenated8Bit { .. } => 5,
            InformationElement::Concatenated16Bit { .. } => 6,
            InformationElement::ApplicationPort8Bit { .. } => 4,
            InformationElement::ApplicationPort16Bit { .. } => 6,
            InformationElement::Unknown { data, .. } => 2 + data.len(),
        }
    }

    fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            InformationElement::Concatenated8Bit {
                reference,
                total_parts,
                sequence,
            } => {
                result.push(0x00);
                result.push(0x03);
                result.push(*reference);
                result.push(*total_parts);
                result.push(*sequence);
            }
            InformationElement::Concatenated16Bit {
                reference,
                total_parts,
                sequence,
            } => {
                result.push(0x08);
                result.push(0x04);
                result.extend_from_slice(&reference.to_be_bytes());
                result.push(*total_parts);
                result.push(*sequence);
            }
            InformationElement::ApplicationPort8Bit {
                destination_port,
                source_port,
            } => {
                result.push(0x04);
                result.push(0x02);
                result.push(*destination_port);
                result.push(*source_port);
            }
            InformationElement::ApplicationPort16Bit {
                destination_port,
                source_port,
            } => {
                result.push(0x05);
                result.push(0x04);
                result.extend_from_slice(&destination_port.to_be_bytes());
                result.extend_from_slice(&source_port.to_be_bytes());
            }
            InformationElement::Unknown { iei, data } => {
                result.push(*iei);
                result.push(data.len() as u8);
                result.extend_from_slice(data);
            }
        }
        result
    }
}

pub fn create_concatenated_udh_8bit(
    reference: u8,
    total_parts: u8,
    sequence: u8,
) -> UserDataHeader {
    UserDataHeader::new(vec![InformationElement::Concatenated8Bit {
        reference,
        total_parts,
        sequence,
    }])
}

pub fn create_concatenated_udh_16bit(
    reference: u16,
    total_parts: u8,
    sequence: u8,
) -> UserDataHeader {
    UserDataHeader::new(vec![InformationElement::Concatenated16Bit {
        reference,
        total_parts,
        sequence,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_8bit_parse_encode() {
        let udh = create_concatenated_udh_8bit(42, 3, 1);
        let encoded = udh.encode();
        let (parsed, _) = UserDataHeader::parse(&encoded).unwrap();

        let concat_info = parsed.get_concat_info().unwrap();
        assert_eq!(concat_info.reference, 42);
        assert_eq!(concat_info.total_parts, 3);
        assert_eq!(concat_info.sequence, 1);
    }

    #[test]
    fn test_concat_16bit_parse_encode() {
        let udh = create_concatenated_udh_16bit(1234, 5, 2);
        let encoded = udh.encode();
        let (parsed, _) = UserDataHeader::parse(&encoded).unwrap();

        let concat_info = parsed.get_concat_info().unwrap();
        assert_eq!(concat_info.reference, 1234);
        assert_eq!(concat_info.total_parts, 5);
        assert_eq!(concat_info.sequence, 2);
    }

    #[test]
    fn test_application_port_8bit() {
        let udh = UserDataHeader::new(vec![InformationElement::ApplicationPort8Bit {
            destination_port: 80,
            source_port: 200,
        }]);
        let encoded = udh.encode();
        let (parsed, _) = UserDataHeader::parse(&encoded).unwrap();

        match &parsed.elements[0] {
            InformationElement::ApplicationPort8Bit {
                destination_port,
                source_port,
            } => {
                assert_eq!(*destination_port, 80);
                assert_eq!(*source_port, 200);
            }
            _ => panic!("Wrong IE type"),
        }
    }
}
