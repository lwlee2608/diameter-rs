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

use crate::dictionary;
use crate::error::Error;
use core::fmt;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use self::integer32::Integer32Avp;
use self::ipv4::IPv4Avp;
use self::octetstring::OctetStringAvp;
use self::utf8string::UTF8StringAvp;

const VENDOR_FLAG: u8 = 0x80;
const MANDATORY_FLAG: u8 = 0x40;
const PRIVATE_FLAG: u8 = 0x20;

#[derive(Debug)]
pub struct Avp {
    header: AvpHeader,
    value: AvpValue,
    padding: u8,
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
    Unknown,
    Address,
    AddressIPv4,
    AddressIPv6,
    DiameterIdentity,
    DiameterURI,
    Enumerated,
    Float32,
    Float64,
    Grouped,
    Integer32,
    Integer64,
    OctetString,
    Time,
    Unsigned32,
    Unsigned64,
    UTF8String,
}

#[derive(Debug)]
pub enum AvpValue {
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

impl fmt::Display for AvpValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AvpValue::AddressIPv4(ipv4_avp) => ipv4_avp.fmt(f),
            AvpValue::Integer32(integer32_avp) => integer32_avp.fmt(f),
            AvpValue::UTF8String(utf8_string_avp) => utf8_string_avp.fmt(f),
            AvpValue::OctetString(octet_string_avp) => octet_string_avp.fmt(f),
        }
    }
}

impl AvpValue {
    pub fn length(&self) -> u32 {
        match self {
            AvpValue::AddressIPv4(avp) => avp.length(),
            AvpValue::Integer32(avp) => avp.length(),
            AvpValue::UTF8String(avp) => avp.length(),
            AvpValue::OctetString(avp) => avp.length(),
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            AvpValue::AddressIPv4(_) => "AddressIPv4".to_string(),
            AvpValue::Integer32(_) => "Integer32".to_string(),
            AvpValue::UTF8String(_) => "UTF8String".to_string(),
            AvpValue::OctetString(_) => "OctetString".to_string(),
        }
    }
}

impl AvpHeader {
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<AvpHeader, Error> {
        let mut b = [0; 8];
        reader.read_exact(&mut b)?;

        let code = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);

        let flags = AvpFlags {
            vendor: (b[4] & VENDOR_FLAG) != 0,
            mandatory: (b[4] & MANDATORY_FLAG) != 0,
            private: (b[4] & PRIVATE_FLAG) != 0,
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

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        // Code
        writer.write_all(&self.code.to_be_bytes())?;

        // Flags
        let mut flags: u8 = 0;
        if self.flags.vendor {
            flags |= VENDOR_FLAG;
        }
        if self.flags.mandatory {
            flags |= MANDATORY_FLAG;
        }
        if self.flags.private {
            flags |= PRIVATE_FLAG;
        }
        writer.write_all(&[flags])?;

        // Length
        let length_bytes = &self.length.to_be_bytes()[1..4];
        writer.write_all(length_bytes)?;

        // Vendor ID
        if let Some(vendor_id) = self.vendor_id {
            writer.write_all(&vendor_id.to_be_bytes())?;
        }

        Ok(())
    }
}

impl Avp {
    pub fn new(
        code: u32,
        vendor_id: Option<u32>,
        value: AvpValue,
        mflag: bool,
        pflag: bool,
    ) -> Avp {
        let header_length = if vendor_id.is_some() { 12 } else { 8 };
        let padding = Avp::pad_to_32_bits(value.length());
        let header = AvpHeader {
            code,
            flags: AvpFlags {
                vendor: if vendor_id.is_some() { true } else { false },
                mandatory: mflag,
                private: pflag,
            },
            length: header_length + value.length() + padding as u32,
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

    pub fn get_padding(&self) -> u8 {
        self.padding
    }

    pub fn get_value(&self) -> &AvpValue {
        &self.value
    }

    pub fn decode_from<R: Read + Seek>(reader: &mut R) -> Result<Avp, Error> {
        let header = AvpHeader::decode_from(reader)?;

        let header_length = if header.flags.vendor { 12 } else { 8 };
        let value_length = header.length - header_length;

        let avp_type = dictionary::DEFAULT_DICT
            .get_avp_type(header.code)
            .unwrap_or(&AvpType::Unknown);

        let value = match avp_type {
            AvpType::Integer32 => AvpValue::Integer32(Integer32Avp::decode_from(reader)?),
            AvpType::UTF8String => {
                AvpValue::UTF8String(UTF8StringAvp::decode_from(reader, value_length as usize)?)
            }
            AvpType::OctetString => {
                AvpValue::OctetString(OctetStringAvp::decode_from(reader, value_length as usize)?)
            }
            _ => AvpValue::Integer32(Integer32Avp::decode_from(reader)?),
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
        });
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.header.encode_to(writer)?;

        let _ = match &self.value {
            AvpValue::Integer32(avp) => avp.encode_to(writer),
            AvpValue::UTF8String(avp) => avp.encode_to(writer),
            AvpValue::OctetString(avp) => avp.encode_to(writer),
            _ => Ok(()),
        };

        // Padding
        for _ in 0..self.padding {
            writer.write_all(&[0])?;
        }

        Ok(())
    }

    fn pad_to_32_bits(length: u32) -> u8 {
        ((4 - (length & 0b11)) % 4) as u8
    }

    pub fn get_integer32(&self) -> Option<i32> {
        match &self.value {
            AvpValue::Integer32(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_utf8string(&self) -> Option<&str> {
        match &self.value {
            AvpValue::UTF8String(avp) => Some(avp.value()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_decode_encode_header() {
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

        let mut encoded = Vec::new();
        header.encode_to(&mut encoded).unwrap();
        assert_eq!(encoded, data);
    }

    #[test]
    fn test_decode_encode_header_with_vendor() {
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

        let mut encoded = Vec::new();
        header.encode_to(&mut encoded).unwrap();
        assert_eq!(encoded, data);
    }
}
