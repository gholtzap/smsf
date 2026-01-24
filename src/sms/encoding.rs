use anyhow::{anyhow, Result};

const GSM_7BIT_BASIC: [char; 128] = [
    '@', '£', '$', '¥', 'è', 'é', 'ù', 'ì', 'ò', 'Ç', '\n', 'Ø', 'ø', '\r', 'Å', 'å',
    'Δ', '_', 'Φ', 'Γ', 'Λ', 'Ω', 'Π', 'Ψ', 'Σ', 'Θ', 'Ξ', '\x1B', 'Æ', 'æ', 'ß', 'É',
    ' ', '!', '"', '#', '¤', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?',
    '¡', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O',
    'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'Ä', 'Ö', 'Ñ', 'Ü', '§',
    '¿', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
    'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'ä', 'ö', 'ñ', 'ü', 'à',
];

const GSM_7BIT_EXT: [(char, u8); 10] = [
    ('|', 0x40),
    ('^', 0x14),
    ('€', 0x65),
    ('{', 0x28),
    ('}', 0x29),
    ('[', 0x3C),
    (']', 0x3E),
    ('~', 0x3D),
    ('\\', 0x2F),
    ('\u{000C}', 0x0A),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DataCodingScheme {
    Gsm7Bit,
    Data8Bit,
    Ucs2,
}

impl DataCodingScheme {
    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x00 => DataCodingScheme::Gsm7Bit,
            0x04 => DataCodingScheme::Data8Bit,
            0x08 => DataCodingScheme::Ucs2,
            _ => DataCodingScheme::Gsm7Bit,
        }
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            DataCodingScheme::Gsm7Bit => 0x00,
            DataCodingScheme::Data8Bit => 0x04,
            DataCodingScheme::Ucs2 => 0x08,
        }
    }
}

pub fn encode_gsm7(text: &str) -> Vec<u8> {
    let mut septets = Vec::new();

    for ch in text.chars() {
        if let Some(pos) = GSM_7BIT_BASIC.iter().position(|&c| c == ch) {
            septets.push(pos as u8);
        } else if let Some((_, code)) = GSM_7BIT_EXT.iter().find(|(c, _)| *c == ch) {
            septets.push(0x1B);
            septets.push(*code);
        } else {
            septets.push(0x3F);
        }
    }

    pack_septets(&septets)
}

pub fn decode_gsm7(data: &[u8], length: usize) -> Result<String> {
    let septets = unpack_septets(data, length);
    let mut text = String::new();
    let mut i = 0;

    while i < septets.len() {
        if septets[i] == 0x1B && i + 1 < septets.len() {
            if let Some((ch, _)) = GSM_7BIT_EXT.iter().find(|(_, code)| *code == septets[i + 1]) {
                text.push(*ch);
            } else {
                text.push('?');
            }
            i += 2;
        } else {
            let idx = septets[i] as usize;
            if idx < GSM_7BIT_BASIC.len() {
                text.push(GSM_7BIT_BASIC[idx]);
            } else {
                text.push('?');
            }
            i += 1;
        }
    }

    Ok(text)
}

fn pack_septets(septets: &[u8]) -> Vec<u8> {
    let mut packed = Vec::new();
    let mut bits = 0;
    let mut value = 0u16;

    for &septet in septets {
        value |= (septet as u16) << bits;
        bits += 7;

        while bits >= 8 {
            packed.push((value & 0xFF) as u8);
            value >>= 8;
            bits -= 8;
        }
    }

    if bits > 0 {
        packed.push((value & 0xFF) as u8);
    }

    packed
}

fn unpack_septets(data: &[u8], length: usize) -> Vec<u8> {
    let mut septets = Vec::new();
    let mut bits = 0;
    let mut value = 0u16;

    for &byte in data {
        value |= (byte as u16) << bits;
        bits += 8;

        while bits >= 7 && septets.len() < length {
            septets.push((value & 0x7F) as u8);
            value >>= 7;
            bits -= 7;
        }

        if septets.len() >= length {
            break;
        }
    }

    septets
}

pub fn encode_ucs2(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .flat_map(|c| c.to_be_bytes())
        .collect()
}

pub fn decode_ucs2(data: &[u8]) -> Result<String> {
    if data.len() % 2 != 0 {
        return Err(anyhow!("Invalid UCS-2 data length"));
    }

    let mut chars = Vec::new();
    for chunk in data.chunks_exact(2) {
        let code = u16::from_be_bytes([chunk[0], chunk[1]]);
        chars.push(code);
    }

    String::from_utf16(&chars).map_err(|e| anyhow!("UCS-2 decode error: {}", e))
}

pub fn auto_detect_encoding(text: &str) -> DataCodingScheme {
    for ch in text.chars() {
        if !GSM_7BIT_BASIC.contains(&ch) && !GSM_7BIT_EXT.iter().any(|(c, _)| *c == ch) {
            return DataCodingScheme::Ucs2;
        }
    }
    DataCodingScheme::Gsm7Bit
}

pub fn encode_text(text: &str, dcs: DataCodingScheme) -> Vec<u8> {
    match dcs {
        DataCodingScheme::Gsm7Bit => encode_gsm7(text),
        DataCodingScheme::Data8Bit => text.as_bytes().to_vec(),
        DataCodingScheme::Ucs2 => encode_ucs2(text),
    }
}

pub fn decode_text(data: &[u8], dcs: DataCodingScheme, length: usize) -> Result<String> {
    match dcs {
        DataCodingScheme::Gsm7Bit => decode_gsm7(data, length),
        DataCodingScheme::Data8Bit => {
            String::from_utf8(data.to_vec()).map_err(|e| anyhow!("UTF-8 decode error: {}", e))
        }
        DataCodingScheme::Ucs2 => decode_ucs2(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gsm7_encoding() {
        let text = "Hello";
        let encoded = encode_gsm7(text);
        let decoded = decode_gsm7(&encoded, text.len()).unwrap();
        assert_eq!(text, decoded);
    }

    #[test]
    fn test_gsm7_with_extended() {
        let text = "Test € message";
        let encoded = encode_gsm7(text);
        let septet_count = text.chars().map(|c| {
            if GSM_7BIT_EXT.iter().any(|(ch, _)| *ch == c) {
                2
            } else {
                1
            }
        }).sum();
        let decoded = decode_gsm7(&encoded, septet_count).unwrap();
        assert_eq!(text, decoded);
    }

    #[test]
    fn test_ucs2_encoding() {
        let text = "Hello 世界";
        let encoded = encode_ucs2(text);
        let decoded = decode_ucs2(&encoded).unwrap();
        assert_eq!(text, decoded);
    }

    #[test]
    fn test_auto_detect() {
        assert_eq!(auto_detect_encoding("Hello"), DataCodingScheme::Gsm7Bit);
        assert_eq!(auto_detect_encoding("Hello 世界"), DataCodingScheme::Ucs2);
    }
}
