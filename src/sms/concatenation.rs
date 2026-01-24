use super::encoding::{encode_text, DataCodingScheme};
use super::tpdu::{TpDeliver, TpSubmit};
use super::udh::{create_concatenated_udh_16bit, create_concatenated_udh_8bit};
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicU16, Ordering};

static CONCAT_REF_COUNTER: AtomicU16 = AtomicU16::new(0);

fn get_next_concat_ref() -> u16 {
    CONCAT_REF_COUNTER.fetch_add(1, Ordering::SeqCst)
}

const MAX_SINGLE_SMS_7BIT: usize = 160;
const MAX_SINGLE_SMS_8BIT: usize = 140;
const MAX_SINGLE_SMS_UCS2: usize = 70;

const MAX_CONCAT_SMS_7BIT: usize = 153;
const MAX_CONCAT_SMS_8BIT: usize = 134;
const MAX_CONCAT_SMS_UCS2: usize = 67;

pub fn split_long_message_submit(
    destination: String,
    text: String,
) -> Result<Vec<TpSubmit>> {
    let dcs = super::encoding::auto_detect_encoding(&text);

    let (max_single, max_concat) = match dcs {
        DataCodingScheme::Gsm7Bit => (MAX_SINGLE_SMS_7BIT, MAX_CONCAT_SMS_7BIT),
        DataCodingScheme::Data8Bit => (MAX_SINGLE_SMS_8BIT, MAX_CONCAT_SMS_8BIT),
        DataCodingScheme::Ucs2 => (MAX_SINGLE_SMS_UCS2, MAX_CONCAT_SMS_UCS2),
    };

    let text_length = match dcs {
        DataCodingScheme::Gsm7Bit => text.len(),
        DataCodingScheme::Ucs2 => text.chars().count(),
        DataCodingScheme::Data8Bit => text.as_bytes().len(),
    };

    if text_length <= max_single {
        let submit = TpSubmit::new(destination, text);
        return Ok(vec![submit]);
    }

    let total_parts = ((text_length + max_concat - 1) / max_concat).min(255) as u8;
    if total_parts > 255 {
        return Err(anyhow!("Message too long to split into parts"));
    }

    let concat_ref = get_next_concat_ref();
    let use_16bit = concat_ref > 255;

    let mut parts = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    for part_num in 0..total_parts {
        let start = (part_num as usize) * max_concat;
        let end = ((part_num + 1) as usize * max_concat).min(chars.len());
        let part_text: String = chars[start..end].iter().collect();

        let user_data = encode_text(&part_text, dcs);
        let user_data_length = match dcs {
            DataCodingScheme::Gsm7Bit => part_text.len() as u8,
            _ => user_data.len() as u8,
        };

        let udh = if use_16bit {
            create_concatenated_udh_16bit(concat_ref, total_parts, part_num + 1)
        } else {
            create_concatenated_udh_8bit(concat_ref as u8, total_parts, part_num + 1)
        };

        let submit = TpSubmit {
            message_reference: 0,
            destination_address: destination.clone(),
            protocol_identifier: 0,
            data_coding_scheme: dcs,
            validity_period: Some(0xAA),
            user_data,
            user_data_length,
            status_report_request: false,
            reply_path: false,
            reject_duplicates: false,
            udh: Some(udh),
        };

        parts.push(submit);
    }

    Ok(parts)
}

pub fn split_long_message_deliver(
    originator: String,
    text: String,
) -> Result<Vec<TpDeliver>> {
    let dcs = super::encoding::auto_detect_encoding(&text);

    let (max_single, max_concat) = match dcs {
        DataCodingScheme::Gsm7Bit => (MAX_SINGLE_SMS_7BIT, MAX_CONCAT_SMS_7BIT),
        DataCodingScheme::Data8Bit => (MAX_SINGLE_SMS_8BIT, MAX_CONCAT_SMS_8BIT),
        DataCodingScheme::Ucs2 => (MAX_SINGLE_SMS_UCS2, MAX_CONCAT_SMS_UCS2),
    };

    let text_length = match dcs {
        DataCodingScheme::Gsm7Bit => text.len(),
        DataCodingScheme::Ucs2 => text.chars().count(),
        DataCodingScheme::Data8Bit => text.as_bytes().len(),
    };

    if text_length <= max_single {
        let deliver = TpDeliver::new(originator, text);
        return Ok(vec![deliver]);
    }

    let total_parts = ((text_length + max_concat - 1) / max_concat).min(255) as u8;
    if total_parts > 255 {
        return Err(anyhow!("Message too long to split into parts"));
    }

    let concat_ref = get_next_concat_ref();
    let use_16bit = concat_ref > 255;

    let mut parts = Vec::new();
    let chars: Vec<char> = text.chars().collect();

    for part_num in 0..total_parts {
        let start = (part_num as usize) * max_concat;
        let end = ((part_num + 1) as usize * max_concat).min(chars.len());
        let part_text: String = chars[start..end].iter().collect();

        let user_data = encode_text(&part_text, dcs);
        let user_data_length = match dcs {
            DataCodingScheme::Gsm7Bit => part_text.len() as u8,
            _ => user_data.len() as u8,
        };

        let udh = if use_16bit {
            create_concatenated_udh_16bit(concat_ref, total_parts, part_num + 1)
        } else {
            create_concatenated_udh_8bit(concat_ref as u8, total_parts, part_num + 1)
        };

        let mut deliver = TpDeliver::new(originator.clone(), part_text);
        deliver.user_data = user_data;
        deliver.user_data_length = user_data_length;
        deliver.udh = Some(udh);

        parts.push(deliver);
    }

    Ok(parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_message_not_split() {
        let dest = "+1234567890".to_string();
        let text = "Short message".to_string();
        let parts = split_long_message_submit(dest, text).unwrap();
        assert_eq!(parts.len(), 1);
        assert!(parts[0].udh.is_none());
    }

    #[test]
    fn test_long_message_split() {
        let dest = "+1234567890".to_string();
        let text = "A".repeat(200);
        let parts = split_long_message_submit(dest, text).unwrap();
        assert!(parts.len() > 1);
        assert!(parts[0].udh.is_some());

        if let Some(concat_info) = parts[0].udh.as_ref().unwrap().get_concat_info() {
            assert_eq!(concat_info.total_parts, parts.len() as u8);
            assert_eq!(concat_info.sequence, 1);
        } else {
            panic!("Expected concat info");
        }
    }

    #[test]
    fn test_unicode_message_split() {
        let dest = "+1234567890".to_string();
        let text = "测试消息".repeat(30);
        let parts = split_long_message_submit(dest, text).unwrap();
        assert!(parts.len() > 1);
        assert!(parts[0].udh.is_some());
    }

    #[test]
    fn test_deliver_split() {
        let orig = "+9876543210".to_string();
        let text = "B".repeat(200);
        let parts = split_long_message_deliver(orig, text).unwrap();
        assert!(parts.len() > 1);
        assert!(parts[0].udh.is_some());
    }
}
