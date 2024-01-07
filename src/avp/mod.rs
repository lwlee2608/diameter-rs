/*
 * The AVP header.
 *
 * AVP header format:
 *   0                   1                   2                   3
 *   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                         Command-Code                          |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |  Flags       |                 AVP Length                     |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                         Vendor ID (optional)                  |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *
 * AVP Flags:
 *   0 1 2 3 4 5 6 7
 *  +-+-+-+-+-+-+-+-+  V(endor), M(andatory), P(rivate)
 *  |V M P r r r r r|  r(eserved)
 *  +-+-+-+-+-+-+-+-+
 *
 */

pub mod address;
pub mod integer32;
pub mod ipv4;
pub mod octetstring;
pub mod utf8string;

use crate::error::Error;
use core::fmt;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use self::integer32::Integer32Avp;
use self::ipv4::IPv4Avp;
use self::octetstring::OctetStringAvp;
use self::utf8string::UTF8StringAvp;

#[derive(Debug)]
pub struct Avp {
    header: AvpHeader,
    value: AvpType,
    padding: u32,
}

#[derive(Debug)]
pub struct AvpHeader {
    code: u32,
    flags: AvpFlags,
    length: u32,
    vendor_id: Option<u32>,
}

#[derive(Debug)]
pub struct AvpFlags {
    pub vendor: bool,
    pub mandatory: bool,
    pub private: bool,
}

#[derive(Debug)]
pub enum AvpType {
    // Address(address::AddressAvp),
    AddressIPv4(IPv4Avp),
    // AddressIPv6,
    // DiameterIdentity,
    // DiameterURI,
    // Enumerated,
    // Float32,
    // Float64,
    // Grouped,
    Integer32(Integer32Avp),
    // Integer64,
    OctetString(OctetStringAvp),
    // Time,
    // Unsigned32,
    // Unsigned64,
    UTF8String(UTF8StringAvp),
}

impl fmt::Display for AvpType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AvpType::AddressIPv4(ipv4_avp) => ipv4_avp.fmt(f),
            AvpType::Integer32(integer32_avp) => integer32_avp.fmt(f),
            AvpType::UTF8String(utf8_string_avp) => utf8_string_avp.fmt(f),
            AvpType::OctetString(octet_string_avp) => octet_string_avp.fmt(f),
        }
    }
}

impl AvpType {
    pub fn length(&self) -> u32 {
        match self {
            AvpType::AddressIPv4(avp) => avp.length(),
            AvpType::Integer32(avp) => avp.length(),
            AvpType::UTF8String(avp) => avp.length(),
            AvpType::OctetString(avp) => avp.length(),
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            AvpType::AddressIPv4(_) => "AddressIPv4".to_string(),
            AvpType::Integer32(_) => "Integer32".to_string(),
            AvpType::UTF8String(_) => "UTF8String".to_string(),
            AvpType::OctetString(_) => "OctetString".to_string(),
        }
    }
}

pub trait AvpData: std::fmt::Debug + std::fmt::Display {
    fn serialize(&self) -> Vec<u8>;
    fn length(&self) -> u32;
}

impl AvpHeader {
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<AvpHeader, Error> {
        let mut b = [0; 8];
        reader.read_exact(&mut b)?;

        let code = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);

        let flags = AvpFlags {
            vendor: (b[4] & 0x80) != 0,
            mandatory: (b[4] & 0x40) != 0,
            private: (b[4] & 0x20) != 0,
        };

        let length = u32::from_be_bytes([0, b[5], b[6], b[7]]);

        if flags.vendor {
            let mut b = [0; 4];
            reader.read_exact(&mut b)?;
            let vendor_id = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);

            Ok(AvpHeader {
                code,
                flags,
                length,
                vendor_id: Some(vendor_id),
            })
        } else {
            Ok(AvpHeader {
                code,
                flags,
                length,
                vendor_id: None,
            })
        }
    }
}

impl Avp {
    pub fn new(code: u32, vendor_id: Option<u32>, value: AvpType, mflag: bool, pflag: bool) -> Avp {
        let header_length = if vendor_id.is_some() { 12 } else { 8 };
        let padding = Avp::pad_to_32_bits(value.length());
        let header = AvpHeader {
            code,
            flags: AvpFlags {
                vendor: if vendor_id.is_some() { true } else { false },
                mandatory: mflag,
                private: pflag,
            },
            length: header_length + value.length() + padding,
            vendor_id,
        };
        return Avp {
            header,
            value,
            padding,
        };
    }

    pub fn get_code(&self) -> u32 {
        self.header.code
    }

    pub fn get_flags(&self) -> &AvpFlags {
        &self.header.flags
    }

    pub fn get_vendor_id(&self) -> Option<u32> {
        self.header.vendor_id
    }

    pub fn get_length(&self) -> u32 {
        self.header.length
    }

    pub fn get_padding(&self) -> u32 {
        self.padding
    }

    pub fn get_value(&self) -> &AvpType {
        &self.value
    }

    pub fn decode_from<R: Read + Seek>(reader: &mut R) -> Result<Avp, Error> {
        let header = AvpHeader::decode_from(reader)?;

        let header_length = if header.flags.vendor { 12 } else { 8 };
        let value_length = header.length - header_length;

        // Hardcode for now
        let value = match header.code {
            30 => AvpType::UTF8String(UTF8StringAvp::decode_from(reader, value_length as usize)?),
            44 => AvpType::OctetString(OctetStringAvp::decode_from(reader, value_length as usize)?),
            571 => AvpType::Integer32(Integer32Avp::decode_from(reader)?),
            _ => AvpType::Integer32(Integer32Avp::decode_from(reader)?),
        };

        // Skip padding
        let padding = Avp::pad_to_32_bits(value_length);
        if padding > 0 {
            reader.seek(SeekFrom::Current(padding as i64))?;
        }

        return Ok(Avp {
            header,
            value,
            padding,
            // value: Box::new(value),
        });
    }

    fn pad_to_32_bits(length: u32) -> u32 {
        let pad_required = length & 0b11;
        if pad_required != 0 {
            4 - pad_required
        } else {
            0
        }
    }

    pub fn get_integer32(&self) -> Option<i32> {
        match &self.value {
            AvpType::Integer32(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_utf8string(&self) -> Option<&str> {
        match &self.value {
            AvpType::UTF8String(avp) => Some(avp.value()),
            _ => None,
        }
    }
    // pub fn serialize(&self) -> Vec<u8> {
    //     match &self.v {
    //         AvpType::Integer32(avp) => avp.serialize(),
    //         AvpType::UTF8String(avp) => avp.serialize(),
    //         _ => Vec::new(),
    //     }
    // }
    // pub fn deserialize(&self, b: &[u8]) {
    //     match &self.v {
    //         AvpType::Integer32(_) => {
    //             let _avp = Integer32Avp::decode_from(&b).unwrap();
    //         }
    //         AvpType::UTF8String(_) => {
    //             let _avp = UTF8StringAvp::decode_from(&b).unwrap();
    //         }
    //         _ => (),
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_decode_header() {
        let data = [
            0x00, 0x00, 0x00, 0x64, // command code
            0x40, 0x00, 0x00, 0x0C, // flags, length
        ];

        let mut cursor = Cursor::new(&data);
        let header = AvpHeader::decode_from(&mut cursor).unwrap();

        assert_eq!(header.code, 100);
        assert_eq!(header.length, 12);
        assert_eq!(header.flags.vendor, false);
        assert_eq!(header.flags.mandatory, true);
        assert_eq!(header.flags.private, false);
        assert_eq!(header.vendor_id, None);
    }

    #[test]
    fn test_decode_header_with_vendor() {
        let data = [
            0x00, 0x00, 0x00, 0x64, // command code
            0x80, 0x00, 0x00, 0x0C, // flags, length
            0x00, 0x00, 0x00, 0xC8, // vendor_id
        ];

        let mut cursor = Cursor::new(&data);
        let header = AvpHeader::decode_from(&mut cursor).unwrap();

        assert_eq!(header.code, 100);
        assert_eq!(header.length, 12);
        assert_eq!(header.flags.vendor, true);
        assert_eq!(header.flags.mandatory, false);
        assert_eq!(header.flags.private, false);
        assert_eq!(header.vendor_id, Some(200));
    }
}
