use anyhow::{anyhow, Result};
use std::collections::HashMap;
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeOfNumber {
    Unknown,
    International,
    National,
    NetworkSpecific,
    Subscriber,
    Alphanumeric,
    Abbreviated,
}

impl TypeOfNumber {
    pub fn to_byte(&self) -> u8 {
        match self {
            TypeOfNumber::Unknown => 0x00,
            TypeOfNumber::International => 0x01,
            TypeOfNumber::National => 0x02,
            TypeOfNumber::NetworkSpecific => 0x03,
            TypeOfNumber::Subscriber => 0x04,
            TypeOfNumber::Alphanumeric => 0x05,
            TypeOfNumber::Abbreviated => 0x06,
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x01 => TypeOfNumber::International,
            0x02 => TypeOfNumber::National,
            0x03 => TypeOfNumber::NetworkSpecific,
            0x04 => TypeOfNumber::Subscriber,
            0x05 => TypeOfNumber::Alphanumeric,
            0x06 => TypeOfNumber::Abbreviated,
            _ => TypeOfNumber::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberingPlanIdentification {
    Unknown,
    Isdn,
    Data,
    Telex,
    NationalStandard,
    Private,
}

impl NumberingPlanIdentification {
    pub fn to_byte(&self) -> u8 {
        match self {
            NumberingPlanIdentification::Unknown => 0x00,
            NumberingPlanIdentification::Isdn => 0x01,
            NumberingPlanIdentification::Data => 0x03,
            NumberingPlanIdentification::Telex => 0x04,
            NumberingPlanIdentification::NationalStandard => 0x08,
            NumberingPlanIdentification::Private => 0x09,
        }
    }

    pub fn from_byte(byte: u8) -> Self {
        match byte {
            0x01 => NumberingPlanIdentification::Isdn,
            0x03 => NumberingPlanIdentification::Data,
            0x04 => NumberingPlanIdentification::Telex,
            0x08 => NumberingPlanIdentification::NationalStandard,
            0x09 => NumberingPlanIdentification::Private,
            _ => NumberingPlanIdentification::Unknown,
        }
    }
}

pub fn encode_ton_npi(ton: TypeOfNumber, npi: NumberingPlanIdentification) -> u8 {
    0x80 | ((ton.to_byte() & 0x07) << 4) | (npi.to_byte() & 0x0F)
}

pub fn decode_ton_npi(byte: u8) -> (TypeOfNumber, NumberingPlanIdentification) {
    let ton = TypeOfNumber::from_byte((byte >> 4) & 0x07);
    let npi = NumberingPlanIdentification::from_byte(byte & 0x0F);
    (ton, npi)
}

#[derive(Debug, Clone)]
pub struct E164Number {
    pub country_code: String,
    pub national_number: String,
    pub full_number: String,
}

lazy_static! {
    static ref COUNTRY_CODE_LENGTHS: HashMap<&'static str, usize> = {
        let mut m = HashMap::new();
        m.insert("1", 1);
        m.insert("7", 1);
        m.insert("20", 2);
        m.insert("27", 2);
        m.insert("30", 2);
        m.insert("31", 2);
        m.insert("32", 2);
        m.insert("33", 2);
        m.insert("34", 2);
        m.insert("36", 2);
        m.insert("39", 2);
        m.insert("40", 2);
        m.insert("41", 2);
        m.insert("43", 2);
        m.insert("44", 2);
        m.insert("45", 2);
        m.insert("46", 2);
        m.insert("47", 2);
        m.insert("48", 2);
        m.insert("49", 2);
        m.insert("51", 2);
        m.insert("52", 2);
        m.insert("53", 2);
        m.insert("54", 2);
        m.insert("55", 2);
        m.insert("56", 2);
        m.insert("57", 2);
        m.insert("58", 2);
        m.insert("60", 2);
        m.insert("61", 2);
        m.insert("62", 2);
        m.insert("63", 2);
        m.insert("64", 2);
        m.insert("65", 2);
        m.insert("66", 2);
        m.insert("81", 2);
        m.insert("82", 2);
        m.insert("84", 2);
        m.insert("86", 2);
        m.insert("90", 2);
        m.insert("91", 2);
        m.insert("92", 2);
        m.insert("93", 2);
        m.insert("94", 2);
        m.insert("95", 2);
        m.insert("98", 2);
        m.insert("212", 3);
        m.insert("213", 3);
        m.insert("216", 3);
        m.insert("218", 3);
        m.insert("220", 3);
        m.insert("221", 3);
        m.insert("222", 3);
        m.insert("223", 3);
        m.insert("224", 3);
        m.insert("225", 3);
        m.insert("226", 3);
        m.insert("227", 3);
        m.insert("228", 3);
        m.insert("229", 3);
        m.insert("230", 3);
        m.insert("231", 3);
        m.insert("232", 3);
        m.insert("233", 3);
        m.insert("234", 3);
        m.insert("235", 3);
        m.insert("236", 3);
        m.insert("237", 3);
        m.insert("238", 3);
        m.insert("239", 3);
        m.insert("240", 3);
        m.insert("241", 3);
        m.insert("242", 3);
        m.insert("243", 3);
        m.insert("244", 3);
        m.insert("245", 3);
        m.insert("246", 3);
        m.insert("248", 3);
        m.insert("249", 3);
        m.insert("250", 3);
        m.insert("251", 3);
        m.insert("252", 3);
        m.insert("253", 3);
        m.insert("254", 3);
        m.insert("255", 3);
        m.insert("256", 3);
        m.insert("257", 3);
        m.insert("258", 3);
        m.insert("260", 3);
        m.insert("261", 3);
        m.insert("262", 3);
        m.insert("263", 3);
        m.insert("264", 3);
        m.insert("265", 3);
        m.insert("266", 3);
        m.insert("267", 3);
        m.insert("268", 3);
        m.insert("269", 3);
        m.insert("290", 3);
        m.insert("291", 3);
        m.insert("297", 3);
        m.insert("298", 3);
        m.insert("299", 3);
        m.insert("350", 3);
        m.insert("351", 3);
        m.insert("352", 3);
        m.insert("353", 3);
        m.insert("354", 3);
        m.insert("355", 3);
        m.insert("356", 3);
        m.insert("357", 3);
        m.insert("358", 3);
        m.insert("359", 3);
        m.insert("370", 3);
        m.insert("371", 3);
        m.insert("372", 3);
        m.insert("373", 3);
        m.insert("374", 3);
        m.insert("375", 3);
        m.insert("376", 3);
        m.insert("377", 3);
        m.insert("378", 3);
        m.insert("380", 3);
        m.insert("381", 3);
        m.insert("382", 3);
        m.insert("383", 3);
        m.insert("385", 3);
        m.insert("386", 3);
        m.insert("387", 3);
        m.insert("389", 3);
        m.insert("420", 3);
        m.insert("421", 3);
        m.insert("423", 3);
        m.insert("500", 3);
        m.insert("501", 3);
        m.insert("502", 3);
        m.insert("503", 3);
        m.insert("504", 3);
        m.insert("505", 3);
        m.insert("506", 3);
        m.insert("507", 3);
        m.insert("508", 3);
        m.insert("509", 3);
        m.insert("590", 3);
        m.insert("591", 3);
        m.insert("592", 3);
        m.insert("593", 3);
        m.insert("594", 3);
        m.insert("595", 3);
        m.insert("596", 3);
        m.insert("597", 3);
        m.insert("598", 3);
        m.insert("599", 3);
        m.insert("670", 3);
        m.insert("672", 3);
        m.insert("673", 3);
        m.insert("674", 3);
        m.insert("675", 3);
        m.insert("676", 3);
        m.insert("677", 3);
        m.insert("678", 3);
        m.insert("679", 3);
        m.insert("680", 3);
        m.insert("681", 3);
        m.insert("682", 3);
        m.insert("683", 3);
        m.insert("685", 3);
        m.insert("686", 3);
        m.insert("687", 3);
        m.insert("688", 3);
        m.insert("689", 3);
        m.insert("690", 3);
        m.insert("691", 3);
        m.insert("692", 3);
        m.insert("850", 3);
        m.insert("852", 3);
        m.insert("853", 3);
        m.insert("855", 3);
        m.insert("856", 3);
        m.insert("880", 3);
        m.insert("886", 3);
        m.insert("960", 3);
        m.insert("961", 3);
        m.insert("962", 3);
        m.insert("963", 3);
        m.insert("964", 3);
        m.insert("965", 3);
        m.insert("966", 3);
        m.insert("967", 3);
        m.insert("968", 3);
        m.insert("970", 3);
        m.insert("971", 3);
        m.insert("972", 3);
        m.insert("973", 3);
        m.insert("974", 3);
        m.insert("975", 3);
        m.insert("976", 3);
        m.insert("977", 3);
        m.insert("992", 3);
        m.insert("993", 3);
        m.insert("994", 3);
        m.insert("995", 3);
        m.insert("996", 3);
        m.insert("998", 3);
        m
    };
}

impl E164Number {
    pub fn parse(number: &str) -> Result<Self> {
        let digits: String = number.chars().filter(|c| c.is_ascii_digit()).collect();

        if digits.is_empty() {
            return Err(anyhow!("Number contains no digits"));
        }

        if digits.len() > 15 {
            return Err(anyhow!("E.164 number cannot exceed 15 digits"));
        }

        let (country_code, national_number) = Self::parse_country_code(&digits)?;

        Ok(E164Number {
            country_code: country_code.to_string(),
            national_number: national_number.to_string(),
            full_number: format!("+{}", digits),
        })
    }

    fn parse_country_code(digits: &str) -> Result<(&str, &str)> {
        for len in (1..=3).rev() {
            if digits.len() > len {
                let potential_cc = &digits[..len];
                if COUNTRY_CODE_LENGTHS.contains_key(potential_cc) {
                    return Ok((potential_cc, &digits[len..]));
                }
            }
        }

        Err(anyhow!("Invalid country code"))
    }

    pub fn is_valid(&self) -> bool {
        !self.country_code.is_empty() &&
        !self.national_number.is_empty() &&
        self.full_number.len() <= 16
    }
}

#[derive(Debug, Clone)]
pub struct Msisdn {
    pub e164: E164Number,
}

impl Msisdn {
    pub fn parse(number: &str) -> Result<Self> {
        let e164 = E164Number::parse(number)?;

        if !e164.is_valid() {
            return Err(anyhow!("Invalid MSISDN format"));
        }

        Ok(Msisdn { e164 })
    }

    pub fn to_international(&self) -> String {
        self.e164.full_number.clone()
    }

    pub fn to_national(&self) -> String {
        self.e164.national_number.clone()
    }

    pub fn country_code(&self) -> &str {
        &self.e164.country_code
    }

    pub fn is_same_country(&self, other: &Msisdn) -> bool {
        self.e164.country_code == other.e164.country_code
    }
}

#[derive(Debug, Clone)]
pub struct ServiceCentreAddress {
    pub address: String,
    pub ton: TypeOfNumber,
    pub npi: NumberingPlanIdentification,
}

impl ServiceCentreAddress {
    pub fn new(address: String) -> Result<Self> {
        if address.is_empty() {
            return Err(anyhow!("Service Centre Address cannot be empty"));
        }

        let (ton, npi) = if address.starts_with('+') || address.chars().all(|c| c.is_ascii_digit()) {
            (TypeOfNumber::International, NumberingPlanIdentification::Isdn)
        } else if address.chars().any(|c| c.is_ascii_alphabetic()) {
            (TypeOfNumber::Alphanumeric, NumberingPlanIdentification::Unknown)
        } else {
            (TypeOfNumber::Unknown, NumberingPlanIdentification::Isdn)
        };

        Ok(ServiceCentreAddress { address, ton, npi })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut result = Vec::new();

        if self.address.is_empty() {
            result.push(0x00);
            return result;
        }

        let digits: String = self.address.chars().filter(|c| c.is_ascii_digit()).collect();

        let ton_npi = encode_ton_npi(self.ton.clone(), self.npi.clone());

        let mut packed_digits = Vec::new();
        let digit_bytes: Vec<u8> = digits.bytes().map(|b| b - b'0').collect();

        for chunk in digit_bytes.chunks(2) {
            if chunk.len() == 2 {
                packed_digits.push((chunk[1] << 4) | chunk[0]);
            } else {
                packed_digits.push(0xF0 | chunk[0]);
            }
        }

        let sca_len = 1 + packed_digits.len();
        result.push(sca_len as u8);
        result.push(ton_npi);
        result.extend_from_slice(&packed_digits);

        result
    }

    pub fn decode(data: &[u8]) -> Result<(Self, usize)> {
        if data.is_empty() {
            return Err(anyhow!("SCA data is empty"));
        }

        let sca_len = data[0] as usize;

        if sca_len == 0 {
            return Ok((
                ServiceCentreAddress {
                    address: String::new(),
                    ton: TypeOfNumber::Unknown,
                    npi: NumberingPlanIdentification::Unknown,
                },
                1,
            ));
        }

        if data.len() < 1 + sca_len {
            return Err(anyhow!("SCA data truncated"));
        }

        let ton_npi = data[1];
        let (ton, npi) = decode_ton_npi(ton_npi);

        let num_octets = sca_len - 1;
        let mut digits = String::new();

        if ton == TypeOfNumber::International {
            digits.push('+');
        }

        for i in 0..num_octets {
            let byte = data[2 + i];
            let d1 = byte & 0x0F;
            let d2 = (byte >> 4) & 0x0F;

            if d1 <= 9 {
                digits.push((b'0' + d1) as char);
            }
            if d2 <= 9 {
                digits.push((b'0' + d2) as char);
            }
        }

        Ok((
            ServiceCentreAddress {
                address: digits,
                ton,
                npi,
            },
            1 + sca_len,
        ))
    }
}

pub struct SmsRouter {
    default_sca: Option<ServiceCentreAddress>,
    home_network_cc: String,
}

impl SmsRouter {
    pub fn new(home_network_cc: String, default_sca: Option<ServiceCentreAddress>) -> Self {
        SmsRouter {
            default_sca,
            home_network_cc,
        }
    }

    pub fn normalize_number(&self, number: &str) -> Result<Msisdn> {
        if number.starts_with('+') {
            Msisdn::parse(number)
        } else if number.starts_with("00") {
            Msisdn::parse(&format!("+{}", &number[2..]))
        } else if number.chars().all(|c| c.is_ascii_digit()) {
            let full_number = if number.len() >= 10 {
                format!("+{}{}", self.home_network_cc, number)
            } else {
                return Err(anyhow!("National number too short"));
            };
            Msisdn::parse(&full_number)
        } else {
            Err(anyhow!("Invalid number format"))
        }
    }

    pub fn is_international(&self, msisdn: &Msisdn) -> bool {
        msisdn.country_code() != self.home_network_cc
    }

    pub fn is_roaming(&self, msisdn: &Msisdn) -> bool {
        self.is_international(msisdn)
    }

    pub fn get_service_centre(&self, _destination: &Msisdn) -> Option<&ServiceCentreAddress> {
        self.default_sca.as_ref()
    }

    pub fn route_sms(&self, from: &str, to: &str) -> Result<SmsRoute> {
        let from_msisdn = self.normalize_number(from)?;
        let to_msisdn = self.normalize_number(to)?;

        let route_type = if self.is_international(&from_msisdn) || self.is_international(&to_msisdn) {
            RouteType::International
        } else {
            RouteType::Domestic
        };

        Ok(SmsRoute {
            from: from_msisdn,
            to: to_msisdn,
            route_type,
            service_centre: self.default_sca.clone(),
        })
    }
}

#[derive(Debug, Clone)]
pub enum RouteType {
    Domestic,
    International,
}

#[derive(Debug, Clone)]
pub struct SmsRoute {
    pub from: Msisdn,
    pub to: Msisdn,
    pub route_type: RouteType,
    pub service_centre: Option<ServiceCentreAddress>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_e164_parsing_us_number() {
        let num = E164Number::parse("+12125551234").unwrap();
        assert_eq!(num.country_code, "1");
        assert_eq!(num.national_number, "2125551234");
        assert_eq!(num.full_number, "+12125551234");
    }

    #[test]
    fn test_e164_parsing_uk_number() {
        let num = E164Number::parse("+447911123456").unwrap();
        assert_eq!(num.country_code, "44");
        assert_eq!(num.national_number, "7911123456");
    }

    #[test]
    fn test_e164_parsing_german_number() {
        let num = E164Number::parse("+4915112345678").unwrap();
        assert_eq!(num.country_code, "49");
        assert_eq!(num.national_number, "15112345678");
    }

    #[test]
    fn test_e164_parsing_without_plus() {
        let num = E164Number::parse("12125551234").unwrap();
        assert_eq!(num.country_code, "1");
        assert_eq!(num.full_number, "+12125551234");
    }

    #[test]
    fn test_e164_too_long() {
        let result = E164Number::parse("+1234567890123456");
        assert!(result.is_err());
    }

    #[test]
    fn test_msisdn_normalization() {
        let router = SmsRouter::new("1".to_string(), None);

        let msisdn = router.normalize_number("+12125551234").unwrap();
        assert_eq!(msisdn.country_code(), "1");

        let msisdn2 = router.normalize_number("0012125551234").unwrap();
        assert_eq!(msisdn2.to_international(), "+12125551234");
    }

    #[test]
    fn test_sca_encoding_decoding() {
        let sca = ServiceCentreAddress::new("+12125551234".to_string()).unwrap();
        let encoded = sca.encode();
        let (decoded, _len) = ServiceCentreAddress::decode(&encoded).unwrap();

        assert_eq!(sca.address, decoded.address);
    }

    #[test]
    fn test_international_routing() {
        let router = SmsRouter::new("1".to_string(), None);
        let msisdn_us = Msisdn::parse("+12125551234").unwrap();
        let msisdn_uk = Msisdn::parse("+447911123456").unwrap();

        assert!(!router.is_international(&msisdn_us));
        assert!(router.is_international(&msisdn_uk));
    }

    #[test]
    fn test_ton_npi_encoding() {
        let ton_npi = encode_ton_npi(
            TypeOfNumber::International,
            NumberingPlanIdentification::Isdn,
        );
        assert_eq!(ton_npi, 0x91);

        let ton_npi2 = encode_ton_npi(
            TypeOfNumber::National,
            NumberingPlanIdentification::Isdn,
        );
        assert_eq!(ton_npi2, 0xA1);
    }

    #[test]
    fn test_sms_routing() {
        let router = SmsRouter::new("1".to_string(), None);
        let route = router.route_sms("+12125551234", "+447911123456").unwrap();

        assert_eq!(route.from.country_code(), "1");
        assert_eq!(route.to.country_code(), "44");
        assert!(matches!(route.route_type, RouteType::International));
    }
}
