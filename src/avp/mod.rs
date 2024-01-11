/*
 * AVP format:
 *   0                   1                   2                   3
 *   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                         Command-Code                          |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |  Flags       |                 AVP Length                     |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                         Vendor ID (optional)                  |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                             Data                              |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                             Data             |    Padding     |
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
pub mod enumerated;
pub mod float32;
pub mod float64;
pub mod identity;
pub mod integer32;
pub mod integer64;
pub mod ipv4;
pub mod ipv6;
pub mod octetstring;
pub mod time;
pub mod unsigned32;
pub mod unsigned64;
pub mod uri;
pub mod utf8string;

use crate::dictionary;
use crate::error::Error;
use core::fmt;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use self::enumerated::EnumeratedAvp;
use self::float32::Float32Avp;
use self::float64::Float64Avp;
use self::identity::IdentityAvp;
use self::integer32::Integer32Avp;
use self::integer64::Integer64Avp;
use self::ipv4::IPv4Avp;
use self::ipv6::IPv6Avp;
use self::octetstring::OctetStringAvp;
use self::time::TimeAvp;
use self::unsigned32::Unsigned32Avp;
use self::unsigned64::Unsigned64Avp;
use self::uri::DiameterURI;
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
    Identity,
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
    AddressIPv6(IPv6Avp),
    Identity(IdentityAvp),
    DiameterURI(DiameterURI),
    Enumerated(EnumeratedAvp),
    Float32(Float32Avp),
    Float64(Float64Avp),
    // Grouped,
    Integer32(Integer32Avp),
    Integer64(Integer64Avp),
    OctetString(OctetStringAvp),
    Time(TimeAvp),
    Unsigned32(Unsigned32Avp),
    Unsigned64(Unsigned64Avp),
    UTF8String(UTF8StringAvp),
}

impl fmt::Display for AvpValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AvpValue::AddressIPv4(avp) => avp.fmt(f),
            AvpValue::AddressIPv6(avp) => avp.fmt(f),
            AvpValue::Float32(avp) => avp.fmt(f),
            AvpValue::Float64(avp) => avp.fmt(f),
            AvpValue::Enumerated(avp) => avp.fmt(f),
            AvpValue::Integer32(avp) => avp.fmt(f),
            AvpValue::Integer64(avp) => avp.fmt(f),
            AvpValue::Unsigned32(avp) => avp.fmt(f),
            AvpValue::Unsigned64(avp) => avp.fmt(f),
            AvpValue::UTF8String(avp) => avp.fmt(f),
            AvpValue::OctetString(avp) => avp.fmt(f),
            AvpValue::Identity(avp) => avp.fmt(f),
            AvpValue::DiameterURI(avp) => avp.fmt(f),
            AvpValue::Time(avp) => avp.fmt(f),
        }
    }
}

impl AvpValue {
    pub fn length(&self) -> u32 {
        match self {
            AvpValue::AddressIPv4(avp) => avp.length(),
            AvpValue::AddressIPv6(avp) => avp.length(),
            AvpValue::Float32(avp) => avp.length(),
            AvpValue::Float64(avp) => avp.length(),
            AvpValue::Enumerated(avp) => avp.length(),
            AvpValue::Integer32(avp) => avp.length(),
            AvpValue::Integer64(avp) => avp.length(),
            AvpValue::Unsigned32(avp) => avp.length(),
            AvpValue::Unsigned64(avp) => avp.length(),
            AvpValue::UTF8String(avp) => avp.length(),
            AvpValue::OctetString(avp) => avp.length(),
            AvpValue::Identity(avp) => avp.length(),
            AvpValue::DiameterURI(avp) => avp.length(),
            AvpValue::Time(avp) => avp.length(),
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        match self {
            AvpValue::AddressIPv4(_) => "AddressIPv4",
            AvpValue::AddressIPv6(_) => "AddressIPv6",
            AvpValue::Float32(_) => "Float32",
            AvpValue::Float64(_) => "Float64",
            AvpValue::Enumerated(_) => "Enumerated",
            AvpValue::Integer32(_) => "Integer32",
            AvpValue::Integer64(_) => "Integer64",
            AvpValue::Unsigned32(_) => "Unsigned32",
            AvpValue::Unsigned64(_) => "Unsigned64",
            AvpValue::UTF8String(_) => "UTF8String",
            AvpValue::OctetString(_) => "OctetString",
            AvpValue::Identity(_) => "Identity",
            AvpValue::DiameterURI(_) => "DiameterURI",
            AvpValue::Time(_) => "Time",
        }
    }
}

impl From<IdentityAvp> for AvpValue {
    fn from(identity: IdentityAvp) -> Self {
        AvpValue::Identity(identity)
    }
}

impl From<DiameterURI> for AvpValue {
    fn from(uri: DiameterURI) -> Self {
        AvpValue::DiameterURI(uri)
    }
}

impl From<EnumeratedAvp> for AvpValue {
    fn from(enumerated: EnumeratedAvp) -> Self {
        AvpValue::Enumerated(enumerated)
    }
}

impl From<Float32Avp> for AvpValue {
    fn from(float32: Float32Avp) -> Self {
        AvpValue::Float32(float32)
    }
}

impl From<Float64Avp> for AvpValue {
    fn from(float64: Float64Avp) -> Self {
        AvpValue::Float64(float64)
    }
}

impl From<Integer32Avp> for AvpValue {
    fn from(integer32: Integer32Avp) -> Self {
        AvpValue::Integer32(integer32)
    }
}

impl From<Integer64Avp> for AvpValue {
    fn from(integer64: Integer64Avp) -> Self {
        AvpValue::Integer64(integer64)
    }
}

impl From<IPv4Avp> for AvpValue {
    fn from(ipv4: IPv4Avp) -> Self {
        AvpValue::AddressIPv4(ipv4)
    }
}

impl From<IPv6Avp> for AvpValue {
    fn from(ipv6: IPv6Avp) -> Self {
        AvpValue::AddressIPv6(ipv6)
    }
}

impl From<OctetStringAvp> for AvpValue {
    fn from(octetstring: OctetStringAvp) -> Self {
        AvpValue::OctetString(octetstring)
    }
}

impl From<TimeAvp> for AvpValue {
    fn from(time: TimeAvp) -> Self {
        AvpValue::Time(time)
    }
}

impl From<Unsigned32Avp> for AvpValue {
    fn from(unsigned32: Unsigned32Avp) -> Self {
        AvpValue::Unsigned32(unsigned32)
    }
}

impl From<Unsigned64Avp> for AvpValue {
    fn from(unsigned64: Unsigned64Avp) -> Self {
        AvpValue::Unsigned64(unsigned64)
    }
}

impl From<UTF8StringAvp> for AvpValue {
    fn from(utf8string: UTF8StringAvp) -> Self {
        AvpValue::UTF8String(utf8string)
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

        // if avp_type == &AvpType::Unknown {
        // return Err(Error::UnknownAvpCode(header.code));
        // }

        let value = match avp_type {
            AvpType::AddressIPv4 => AvpValue::AddressIPv4(IPv4Avp::decode_from(reader)?),
            AvpType::AddressIPv6 => AvpValue::AddressIPv6(IPv6Avp::decode_from(reader)?),
            AvpType::Float32 => AvpValue::Float32(Float32Avp::decode_from(reader)?),
            AvpType::Float64 => AvpValue::Float64(Float64Avp::decode_from(reader)?),
            AvpType::Enumerated => AvpValue::Enumerated(EnumeratedAvp::decode_from(reader)?),
            AvpType::Integer32 => AvpValue::Integer32(Integer32Avp::decode_from(reader)?),
            AvpType::Integer64 => AvpValue::Integer64(Integer64Avp::decode_from(reader)?),
            AvpType::Unsigned32 => AvpValue::Unsigned32(Unsigned32Avp::decode_from(reader)?),
            AvpType::Unsigned64 => AvpValue::Unsigned64(Unsigned64Avp::decode_from(reader)?),
            AvpType::UTF8String => {
                AvpValue::UTF8String(UTF8StringAvp::decode_from(reader, value_length as usize)?)
            }
            AvpType::OctetString => {
                AvpValue::OctetString(OctetStringAvp::decode_from(reader, value_length as usize)?)
            }
            AvpType::Identity => {
                AvpValue::Identity(IdentityAvp::decode_from(reader, value_length as usize)?)
            }
            AvpType::DiameterURI => {
                AvpValue::DiameterURI(DiameterURI::decode_from(reader, value_length as usize)?)
            }
            AvpType::Time => AvpValue::Time(TimeAvp::decode_from(reader)?),
            AvpType::Unknown => return Err(Error::UnknownAvpCode(header.code)),
            _ => todo!(),
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
            AvpValue::AddressIPv4(avp) => avp.encode_to(writer),
            AvpValue::AddressIPv6(avp) => avp.encode_to(writer),
            AvpValue::Float32(avp) => avp.encode_to(writer),
            AvpValue::Float64(avp) => avp.encode_to(writer),
            AvpValue::Enumerated(avp) => avp.encode_to(writer),
            AvpValue::Integer32(avp) => avp.encode_to(writer),
            AvpValue::Integer64(avp) => avp.encode_to(writer),
            AvpValue::Unsigned32(avp) => avp.encode_to(writer),
            AvpValue::Unsigned64(avp) => avp.encode_to(writer),
            AvpValue::UTF8String(avp) => avp.encode_to(writer),
            AvpValue::OctetString(avp) => avp.encode_to(writer),
            AvpValue::Identity(avp) => avp.encode_to(writer),
            AvpValue::DiameterURI(avp) => avp.encode_to(writer),
            AvpValue::Time(avp) => avp.encode_to(writer),
            // _ => todo!(),
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

#[macro_export]
macro_rules! avp {
    ($code:expr, $vendor_id:expr, $value:expr) => {
        Avp::new($code, $vendor_id, $value.into(), false, false)
    };
    ($code:expr, $vendor_id:expr, $value:expr, $mflag:expr) => {
        Avp::new($code, $vendor_id, $value.into(), $mflag, false)
    };
    ($code:expr, $vendor_id:expr, $value:expr, $mflag:expr, $pflag:expr) => {
        Avp::new($code, $vendor_id, $value.into(), $mflag, $pflag)
    };
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
