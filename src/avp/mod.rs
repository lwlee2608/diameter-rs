//! # AVP Module
//!
//! This module defines the structure and functionalities related to AVPs in Diameter messages.
//!
//! ## AVP Format
//! The diagram below illustrates the format for an AVP:
//! ```text
//!   0                   1                   2                   3
//!   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |                         Command-Code                          |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |  Flags       |                 AVP Length                     |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |                         Vendor ID (optional)                  |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |                             Data                              |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!  |                             Data             |    Padding     |
//!  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
//!
//!  AVP Flags:
//!    0 1 2 3 4 5 6 7
//!   +-+-+-+-+-+-+-+-+  V(endor), M(andatory), P(rivate)
//!   |V M P r r r r r|  r(eserved)
//!   +-+-+-+-+-+-+-+-+
//! ```
//!

pub mod address;
pub mod enumerated;
pub mod float32;
pub mod float64;
pub mod group;
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
use crate::error::{Error, Result};
use core::fmt;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

pub use crate::avp::address::Address;
pub use crate::avp::enumerated::Enumerated;
pub use crate::avp::float32::Float32;
pub use crate::avp::float64::Float64;
pub use crate::avp::group::Grouped;
pub use crate::avp::identity::Identity;
pub use crate::avp::integer32::Integer32;
pub use crate::avp::integer64::Integer64;
pub use crate::avp::ipv4::IPv4;
pub use crate::avp::ipv6::IPv6;
pub use crate::avp::octetstring::OctetString;
pub use crate::avp::time::Time;
pub use crate::avp::unsigned32::Unsigned32;
pub use crate::avp::unsigned64::Unsigned64;
pub use crate::avp::uri::DiameterURI;
pub use crate::avp::utf8string::UTF8String;

pub mod flags {
    pub const V: u8 = 0x80;
    pub const M: u8 = 0x40;
    pub const P: u8 = 0x20;
}

#[derive(Debug, Clone)]
pub struct Avp {
    header: AvpHeader,
    value: AvpValue,
    padding: u8,
}

#[derive(Debug, Clone)]
pub struct AvpHeader {
    code: u32,
    flags: AvpFlags,
    length: u32,
    vendor_id: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct AvpFlags {
    pub vendor: bool,
    pub mandatory: bool,
    pub private: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone)]
pub enum AvpValue {
    Address(Address),
    AddressIPv4(IPv4),
    AddressIPv6(IPv6),
    Identity(Identity),
    DiameterURI(DiameterURI),
    Enumerated(Enumerated),
    Float32(Float32),
    Float64(Float64),
    Grouped(Grouped),
    Integer32(Integer32),
    Integer64(Integer64),
    OctetString(OctetString),
    Time(Time),
    Unsigned32(Unsigned32),
    Unsigned64(Unsigned64),
    UTF8String(UTF8String),
}

impl fmt::Display for AvpValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt(f, 0)
    }
}

impl AvpValue {
    pub fn length(&self) -> u32 {
        match self {
            AvpValue::Address(avp) => avp.length(),
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
            AvpValue::Grouped(avp) => avp.length(),
        }
    }

    pub fn get_type_name(&self) -> &'static str {
        match self {
            AvpValue::Address(_) => "Address",
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
            AvpValue::Grouped(_) => "Grouped",
        }
    }

    fn fmt(&self, f: &mut fmt::Formatter, depth: usize) -> fmt::Result {
        match self {
            AvpValue::Address(avp) => write!(f, "{}", avp),
            AvpValue::AddressIPv4(avp) => write!(f, "{}", avp),
            AvpValue::AddressIPv6(avp) => write!(f, "{}", avp),
            AvpValue::Float32(avp) => write!(f, "{}", avp),
            AvpValue::Float64(avp) => write!(f, "{}", avp),
            AvpValue::Enumerated(avp) => write!(f, "{}", avp),
            AvpValue::Integer32(avp) => write!(f, "{}", avp),
            AvpValue::Integer64(avp) => write!(f, "{}", avp),
            AvpValue::Unsigned32(avp) => write!(f, "{}", avp),
            AvpValue::Unsigned64(avp) => write!(f, "{}", avp),
            AvpValue::UTF8String(avp) => write!(f, "{}", avp),
            AvpValue::OctetString(avp) => write!(f, "{}", avp),
            AvpValue::Identity(avp) => write!(f, "{}", avp),
            AvpValue::DiameterURI(avp) => write!(f, "{}", avp),
            AvpValue::Time(avp) => write!(f, "{}", avp),
            AvpValue::Grouped(avp) => avp.fmt(f, depth),
        }
    }
}

impl From<Identity> for AvpValue {
    fn from(identity: Identity) -> Self {
        AvpValue::Identity(identity)
    }
}

impl From<DiameterURI> for AvpValue {
    fn from(uri: DiameterURI) -> Self {
        AvpValue::DiameterURI(uri)
    }
}

impl From<Enumerated> for AvpValue {
    fn from(enumerated: Enumerated) -> Self {
        AvpValue::Enumerated(enumerated)
    }
}

impl From<Float32> for AvpValue {
    fn from(float32: Float32) -> Self {
        AvpValue::Float32(float32)
    }
}

impl From<Float64> for AvpValue {
    fn from(float64: Float64) -> Self {
        AvpValue::Float64(float64)
    }
}

impl From<Integer32> for AvpValue {
    fn from(integer32: Integer32) -> Self {
        AvpValue::Integer32(integer32)
    }
}

impl From<Integer64> for AvpValue {
    fn from(integer64: Integer64) -> Self {
        AvpValue::Integer64(integer64)
    }
}

impl From<Address> for AvpValue {
    fn from(avp: Address) -> Self {
        AvpValue::Address(avp)
    }
}

impl From<IPv4> for AvpValue {
    fn from(ipv4: IPv4) -> Self {
        AvpValue::AddressIPv4(ipv4)
    }
}

impl From<IPv6> for AvpValue {
    fn from(ipv6: IPv6) -> Self {
        AvpValue::AddressIPv6(ipv6)
    }
}

impl From<OctetString> for AvpValue {
    fn from(octetstring: OctetString) -> Self {
        AvpValue::OctetString(octetstring)
    }
}

impl From<Time> for AvpValue {
    fn from(time: Time) -> Self {
        AvpValue::Time(time)
    }
}

impl From<Unsigned32> for AvpValue {
    fn from(unsigned32: Unsigned32) -> Self {
        AvpValue::Unsigned32(unsigned32)
    }
}

impl From<Unsigned64> for AvpValue {
    fn from(unsigned64: Unsigned64) -> Self {
        AvpValue::Unsigned64(unsigned64)
    }
}

impl From<UTF8String> for AvpValue {
    fn from(utf8string: UTF8String) -> Self {
        AvpValue::UTF8String(utf8string)
    }
}

impl From<Grouped> for AvpValue {
    fn from(group: Grouped) -> Self {
        AvpValue::Grouped(group)
    }
}

impl AvpHeader {
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<AvpHeader> {
        let mut b = [0; 8];
        reader.read_exact(&mut b)?;

        let code = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);

        let flags = AvpFlags {
            vendor: (b[4] & flags::V) != 0,
            mandatory: (b[4] & flags::M) != 0,
            private: (b[4] & flags::P) != 0,
        };

        let length = u32::from_be_bytes([0, b[5], b[6], b[7]]);

        let vendor_id = if flags.vendor {
            let mut b = [0; 4];
            reader.read_exact(&mut b)?;
            Some(u32::from_be_bytes([b[0], b[1], b[2], b[3]]))
        } else {
            None
        };

        Ok(AvpHeader {
            code,
            flags,
            length,
            vendor_id,
        })
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        // Code
        writer.write_all(&self.code.to_be_bytes())?;

        // Flags
        let mut flags: u8 = 0;
        if self.flags.vendor {
            flags |= flags::V;
        }
        if self.flags.mandatory {
            flags |= flags::M;
        }
        if self.flags.private {
            flags |= flags::P;
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
    pub fn new(code: u32, vendor_id: Option<u32>, flags: u8, value: AvpValue) -> Avp {
        let header_length = if vendor_id.is_some() { 12 } else { 8 };
        let padding = Avp::pad_to_32_bits(value.length());
        let header = AvpHeader {
            code,
            flags: AvpFlags {
                vendor: if vendor_id.is_some() { true } else { false },
                mandatory: (flags & flags::M) != 0,
                private: (flags & flags::P) != 0,
            },
            length: header_length + value.length(),
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

    pub fn decode_from<R: Read + Seek>(reader: &mut R) -> Result<Avp> {
        let header = AvpHeader::decode_from(reader)?;

        let header_length = if header.flags.vendor { 12 } else { 8 };
        let value_length = header.length - header_length;

        let dict = dictionary::DEFAULT_DICT.read().unwrap();
        let avp_type = dict
            .get_avp_type(header.code, header.vendor_id)
            .unwrap_or(&AvpType::Unknown);

        let value = match avp_type {
            AvpType::Address => {
                AvpValue::Address(Address::decode_from(reader, value_length as usize)?)
            }
            AvpType::AddressIPv4 => AvpValue::AddressIPv4(IPv4::decode_from(reader)?),
            AvpType::AddressIPv6 => AvpValue::AddressIPv6(IPv6::decode_from(reader)?),
            AvpType::Float32 => AvpValue::Float32(Float32::decode_from(reader)?),
            AvpType::Float64 => AvpValue::Float64(Float64::decode_from(reader)?),
            AvpType::Enumerated => AvpValue::Enumerated(Enumerated::decode_from(reader)?),
            AvpType::Integer32 => AvpValue::Integer32(Integer32::decode_from(reader)?),
            AvpType::Integer64 => AvpValue::Integer64(Integer64::decode_from(reader)?),
            AvpType::Unsigned32 => AvpValue::Unsigned32(Unsigned32::decode_from(reader)?),
            AvpType::Unsigned64 => AvpValue::Unsigned64(Unsigned64::decode_from(reader)?),
            AvpType::UTF8String => {
                AvpValue::UTF8String(UTF8String::decode_from(reader, value_length as usize)?)
            }
            AvpType::OctetString => {
                AvpValue::OctetString(OctetString::decode_from(reader, value_length as usize)?)
            }
            AvpType::Identity => {
                AvpValue::Identity(Identity::decode_from(reader, value_length as usize)?)
            }
            AvpType::DiameterURI => {
                AvpValue::DiameterURI(DiameterURI::decode_from(reader, value_length as usize)?)
            }
            AvpType::Time => AvpValue::Time(Time::decode_from(reader)?),
            AvpType::Grouped => {
                AvpValue::Grouped(Grouped::decode_from(reader, value_length as usize)?)
            }
            AvpType::Unknown => return Err(Error::UnknownAvpCode(header.code)),
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

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.header.encode_to(writer)?;

        let _ = match &self.value {
            AvpValue::Address(avp) => avp.encode_to(writer),
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
            AvpValue::Grouped(avp) => avp.encode_to(writer),
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

    pub fn get_address(&self) -> Option<&Address> {
        match &self.value {
            AvpValue::Address(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_address_ipv4(&self) -> Option<&IPv4> {
        match &self.value {
            AvpValue::AddressIPv4(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_address_ipv6(&self) -> Option<&IPv6> {
        match &self.value {
            AvpValue::AddressIPv6(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_identity(&self) -> Option<&Identity> {
        match &self.value {
            AvpValue::Identity(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_diameter_uri(&self) -> Option<&DiameterURI> {
        match &self.value {
            AvpValue::DiameterURI(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_enumerated(&self) -> Option<&Enumerated> {
        match &self.value {
            AvpValue::Enumerated(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_integer32(&self) -> Option<i32> {
        match &self.value {
            AvpValue::Integer32(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_integer64(&self) -> Option<i64> {
        match &self.value {
            AvpValue::Integer64(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_unsigned32(&self) -> Option<u32> {
        match &self.value {
            AvpValue::Unsigned32(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_unsigned64(&self) -> Option<u64> {
        match &self.value {
            AvpValue::Unsigned64(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_utf8string(&self) -> Option<&UTF8String> {
        match &self.value {
            AvpValue::UTF8String(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_octetstring(&self) -> Option<&OctetString> {
        match &self.value {
            AvpValue::OctetString(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_time(&self) -> Option<&Time> {
        match &self.value {
            AvpValue::Time(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn get_float32(&self) -> Option<f32> {
        match &self.value {
            AvpValue::Float32(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_float64(&self) -> Option<f64> {
        match &self.value {
            AvpValue::Float64(avp) => Some(avp.value()),
            _ => None,
        }
    }

    pub fn get_grouped(&self) -> Option<&Grouped> {
        match &self.value {
            AvpValue::Grouped(avp) => Some(avp),
            _ => None,
        }
    }

    pub fn fmt(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        let indent = "  ".repeat(depth.max(0));

        let dict = dictionary::DEFAULT_DICT.read().unwrap();

        let avp_name = dict
            .get_avp_name(self.get_code() as u32, self.get_vendor_id())
            .unwrap_or("Unknown");

        let avp_name = format!("{}{}", indent, avp_name);

        let vendor_id = match self.get_vendor_id() {
            Some(v) => v.to_string(),
            None => "".to_string(),
        };

        write!(
            f,
            "  {:<40} {:>8} {:>5}  {} {} {}  {:<16}  ",
            avp_name,
            vendor_id,
            self.get_code(),
            get_bool_unicode(self.get_flags().vendor),
            get_bool_unicode(self.get_flags().mandatory),
            get_bool_unicode(self.get_flags().private),
            self.get_value().get_type_name(),
        )?;

        self.get_value().fmt(f, depth)
    }
}

fn get_bool_unicode(v: bool) -> &'static str {
    if v {
        "✓"
    } else {
        "✗"
    }
}

impl fmt::Display for Avp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[macro_export]
macro_rules! avp {
    ($code:expr, $vendor_id:expr, $flags:expr, $value:expr $(,)?) => {
        Avp::new($code, $vendor_id, $flags, $value.into())
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
