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
pub mod utf8string;

use crate::error::Error;
use core::fmt;
use std::io::Read;

use self::integer32::Integer32Avp;

#[derive(Debug)]
pub struct Avp {
    pub header: AvpHeader,
    // pub data: Vec<u8>,
    // pub type_: AvpType,
    // pub value: Box<dyn AvpData>,
    pub value: AvpType,
}

#[derive(Debug)]
pub struct AvpHeader {
    pub code: u32,
    pub flags: AvpFlags,
    pub length: u32,
    pub vendor_id: Option<u32>,
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
    AddressIPv4(ipv4::IPv4Avp),
    // AddressIPv6,
    // DiameterIdentity,
    // DiameterURI,
    // Enumerated,
    // Float32,
    // Float64,
    // Grouped,
    Integer32(integer32::Integer32Avp),
    // Integer64,
    // OctetString,
    // Time,
    // Unsigned32,
    // Unsigned64,
    UTF8String(utf8string::UTF8StringAvp),
}

impl fmt::Display for AvpType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AvpType::AddressIPv4(ipv4_avp) => write!(f, "AddressIPv4: {}", ipv4_avp),
            AvpType::Integer32(integer32_avp) => write!(f, "Integer32: {}", integer32_avp),
            AvpType::UTF8String(utf8_string_avp) => write!(f, "UTF8String: {}", utf8_string_avp),
        }
    }
}

pub trait AvpData: std::fmt::Debug + std::fmt::Display {
    fn serialize(&self) -> Vec<u8>;
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
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Avp, Error> {
        let header = AvpHeader::decode_from(reader)?;
        let value = Integer32Avp::decode_from(reader)?;
        return Ok(Avp {
            header,
            // value: Box::new(value),
            value: AvpType::Integer32(value),
        });
    }

    pub fn get_integer32(&self) -> Option<i32> {
        match &self.value {
            AvpType::Integer32(integer32_avp) => Some(integer32_avp.value()),
            // TODO Handle other variants or return None
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
