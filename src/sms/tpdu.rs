use super::encoding::{decode_text, encode_text, DataCodingScheme};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Timelike, Utc};

#[derive(Debug, Clone)]
pub enum TpPdu {
    Submit(TpSubmit),
    Deliver(TpDeliver),
    StatusReport(TpStatusReport),
}

#[derive(Debug, Clone)]
pub struct TpSubmit {
    pub message_reference: u8,
    pub destination_address: String,
    pub protocol_identifier: u8,
    pub data_coding_scheme: DataCodingScheme,
    pub validity_period: Option<u8>,
    pub user_data: Vec<u8>,
    pub user_data_length: u8,
    pub status_report_request: bool,
    pub reply_path: bool,
    pub reject_duplicates: bool,
}

#[derive(Debug, Clone)]
pub struct TpDeliver {
    pub originating_address: String,
    pub protocol_identifier: u8,
    pub data_coding_scheme: DataCodingScheme,
    pub timestamp: DateTime<Utc>,
    pub user_data: Vec<u8>,
    pub user_data_length: u8,
    pub more_messages_to_send: bool,
    pub status_report_indication: bool,
    pub reply_path: bool,
}

#[derive(Debug, Clone)]
pub struct TpStatusReport {
    pub message_reference: u8,
    pub recipient_address: String,
    pub timestamp: DateTime<Utc>,
    pub discharge_time: DateTime<Utc>,
    pub status: u8,
    pub parameter_indicator: u8,
}

impl TpSubmit {
    pub fn new(destination: String, text: String) -> Self {
        let dcs = super::encoding::auto_detect_encoding(&text);
        let user_data = encode_text(&text, dcs);
        let user_data_length = match dcs {
            DataCodingScheme::Gsm7Bit => text.len() as u8,
            _ => user_data.len() as u8,
        };

        Self {
            message_reference: 0,
            destination_address: destination,
            protocol_identifier: 0,
            data_coding_scheme: dcs,
            validity_period: Some(0xAA),
            user_data,
            user_data_length,
            status_report_request: false,
            reply_path: false,
            reject_duplicates: false,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut pdu = Vec::new();

        let mut mti_flags = 0x01u8;
        if self.reject_duplicates {
            mti_flags |= 0x04;
        }
        if self.validity_period.is_some() {
            mti_flags |= 0x10;
        }
        if self.status_report_request {
            mti_flags |= 0x20;
        }
        if self.reply_path {
            mti_flags |= 0x80;
        }
        pdu.push(mti_flags);

        pdu.push(self.message_reference);

        let dest_addr = encode_address(&self.destination_address);
        pdu.extend_from_slice(&dest_addr);

        pdu.push(self.protocol_identifier);
        pdu.push(self.data_coding_scheme.to_byte());

        if let Some(vp) = self.validity_period {
            pdu.push(vp);
        }

        pdu.push(self.user_data_length);
        pdu.extend_from_slice(&self.user_data);

        pdu
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 7 {
            return Err(anyhow!("TP-SUBMIT PDU too short"));
        }

        let mti_flags = data[0];
        let reject_duplicates = (mti_flags & 0x04) != 0;
        let has_validity_period = (mti_flags & 0x10) != 0;
        let status_report_request = (mti_flags & 0x20) != 0;
        let reply_path = (mti_flags & 0x80) != 0;

        let message_reference = data[1];

        let (destination_address, addr_len) = decode_address(&data[2..])?;
        let mut pos = 2 + addr_len;

        let protocol_identifier = data[pos];
        pos += 1;

        let dcs_byte = data[pos];
        pos += 1;
        let data_coding_scheme = DataCodingScheme::from_byte(dcs_byte);

        let validity_period = if has_validity_period {
            let vp = data[pos];
            pos += 1;
            Some(vp)
        } else {
            None
        };

        let user_data_length = data[pos];
        pos += 1;

        let user_data = data[pos..].to_vec();

        Ok(Self {
            message_reference,
            destination_address,
            protocol_identifier,
            data_coding_scheme,
            validity_period,
            user_data,
            user_data_length,
            status_report_request,
            reply_path,
            reject_duplicates,
        })
    }

    pub fn get_text(&self) -> Result<String> {
        decode_text(&self.user_data, self.data_coding_scheme, self.user_data_length as usize)
    }
}

impl TpDeliver {
    pub fn new(originator: String, text: String) -> Self {
        let dcs = super::encoding::auto_detect_encoding(&text);
        let user_data = encode_text(&text, dcs);
        let user_data_length = match dcs {
            DataCodingScheme::Gsm7Bit => text.len() as u8,
            _ => user_data.len() as u8,
        };

        Self {
            originating_address: originator,
            protocol_identifier: 0,
            data_coding_scheme: dcs,
            timestamp: Utc::now(),
            user_data,
            user_data_length,
            more_messages_to_send: false,
            status_report_indication: false,
            reply_path: false,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut pdu = Vec::new();

        let mut mti_flags = 0x00u8;
        if self.more_messages_to_send {
            mti_flags |= 0x04;
        }
        if self.status_report_indication {
            mti_flags |= 0x20;
        }
        if self.reply_path {
            mti_flags |= 0x80;
        }
        pdu.push(mti_flags);

        let orig_addr = encode_address(&self.originating_address);
        pdu.extend_from_slice(&orig_addr);

        pdu.push(self.protocol_identifier);
        pdu.push(self.data_coding_scheme.to_byte());

        let scts = encode_timestamp(&self.timestamp);
        pdu.extend_from_slice(&scts);

        pdu.push(self.user_data_length);
        pdu.extend_from_slice(&self.user_data);

        pdu
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 14 {
            return Err(anyhow!("TP-DELIVER PDU too short"));
        }

        let mti_flags = data[0];
        let more_messages_to_send = (mti_flags & 0x04) != 0;
        let status_report_indication = (mti_flags & 0x20) != 0;
        let reply_path = (mti_flags & 0x80) != 0;

        let (originating_address, addr_len) = decode_address(&data[1..])?;
        let mut pos = 1 + addr_len;

        let protocol_identifier = data[pos];
        pos += 1;

        let dcs_byte = data[pos];
        pos += 1;
        let data_coding_scheme = DataCodingScheme::from_byte(dcs_byte);

        let timestamp = decode_timestamp(&data[pos..pos + 7])?;
        pos += 7;

        let user_data_length = data[pos];
        pos += 1;

        let user_data = data[pos..].to_vec();

        Ok(Self {
            originating_address,
            protocol_identifier,
            data_coding_scheme,
            timestamp,
            user_data,
            user_data_length,
            more_messages_to_send,
            status_report_indication,
            reply_path,
        })
    }

    pub fn get_text(&self) -> Result<String> {
        decode_text(&self.user_data, self.data_coding_scheme, self.user_data_length as usize)
    }
}

fn encode_address(address: &str) -> Vec<u8> {
    let mut result = Vec::new();

    let digits: String = address.chars().filter(|c| c.is_ascii_digit()).collect();
    let addr_len = digits.len();

    result.push(addr_len as u8);

    let ton_npi = if address.starts_with('+') {
        0x91
    } else {
        0x81
    };
    result.push(ton_npi);

    let mut packed_digits = Vec::new();
    let digit_bytes: Vec<u8> = digits.bytes().map(|b| b - b'0').collect();

    for chunk in digit_bytes.chunks(2) {
        if chunk.len() == 2 {
            packed_digits.push((chunk[1] << 4) | chunk[0]);
        } else {
            packed_digits.push(0xF0 | chunk[0]);
        }
    }

    result.extend_from_slice(&packed_digits);
    result
}

fn decode_address(data: &[u8]) -> Result<(String, usize)> {
    if data.len() < 2 {
        return Err(anyhow!("Address data too short"));
    }

    let addr_len = data[0] as usize;
    let ton_npi = data[1];

    let num_octets = (addr_len + 1) / 2;
    if data.len() < 2 + num_octets {
        return Err(anyhow!("Address data truncated"));
    }

    let mut digits = String::new();
    if ton_npi == 0x91 {
        digits.push('+');
    }

    for i in 0..num_octets {
        let byte = data[2 + i];
        let d1 = byte & 0x0F;
        let d2 = (byte >> 4) & 0x0F;

        digits.push((b'0' + d1) as char);
        if d2 != 0x0F {
            digits.push((b'0' + d2) as char);
        }
    }

    Ok((digits, 2 + num_octets))
}

fn encode_timestamp(dt: &DateTime<Utc>) -> Vec<u8> {
    let mut scts = Vec::new();

    let year = (dt.year() % 100) as u8;
    scts.push(to_bcd(year));

    scts.push(to_bcd(dt.month() as u8));
    scts.push(to_bcd(dt.day() as u8));
    scts.push(to_bcd(dt.hour() as u8));
    scts.push(to_bcd(dt.minute() as u8));
    scts.push(to_bcd(dt.second() as u8));

    scts.push(0x00);

    scts
}

fn decode_timestamp(data: &[u8]) -> Result<DateTime<Utc>> {
    if data.len() < 7 {
        return Err(anyhow!("Timestamp data too short"));
    }

    let year = from_bcd(data[0]) as i32 + 2000;
    let month = from_bcd(data[1]) as u32;
    let day = from_bcd(data[2]) as u32;
    let hour = from_bcd(data[3]) as u32;
    let minute = from_bcd(data[4]) as u32;
    let second = from_bcd(data[5]) as u32;

    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|d| d.and_hms_opt(hour, minute, second))
        .and_then(|dt| dt.and_local_timezone(Utc).single())
        .ok_or_else(|| anyhow!("Invalid timestamp"))
}

fn to_bcd(value: u8) -> u8 {
    let tens = value / 10;
    let ones = value % 10;
    (ones << 4) | tens
}

fn from_bcd(byte: u8) -> u8 {
    let tens = byte & 0x0F;
    let ones = (byte >> 4) & 0x0F;
    (tens * 10) + ones
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tp_submit_encode_decode() {
        let submit = TpSubmit::new("+1234567890".to_string(), "Hello".to_string());
        let encoded = submit.encode();
        let decoded = TpSubmit::decode(&encoded).unwrap();

        assert_eq!(submit.destination_address, decoded.destination_address);
        assert_eq!(submit.get_text().unwrap(), decoded.get_text().unwrap());
    }

    #[test]
    fn test_tp_deliver_encode_decode() {
        let deliver = TpDeliver::new("+9876543210".to_string(), "Test message".to_string());
        let encoded = deliver.encode();
        let decoded = TpDeliver::decode(&encoded).unwrap();

        assert_eq!(deliver.originating_address, decoded.originating_address);
        assert_eq!(deliver.get_text().unwrap(), decoded.get_text().unwrap());
    }

    #[test]
    fn test_address_encoding() {
        let addr = "+1234567890";
        let encoded = encode_address(addr);
        let (decoded, _) = decode_address(&encoded).unwrap();
        assert_eq!(addr, decoded);
    }
}
