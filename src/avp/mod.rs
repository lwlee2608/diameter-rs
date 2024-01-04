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

#[derive(Debug)]
pub struct Avp {
    pub header: AvpHeader,
    // pub data: Vec<u8>,
    // pub type_: AvpType,
    pub value: Box<dyn AvpData>,
    pub v: AvpType,
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

pub trait AvpData: std::fmt::Debug + std::fmt::Display {
    fn serialize(&self) -> Vec<u8>;
}

impl Avp {
    // pub fn decode_from(b: &[u8]) -> Avp {
    //     return Avp::new(b);
    // }

    pub fn serialize(&self) -> Vec<u8> {
        match &self.v {
            AvpType::Integer32(avp) => avp.serialize(),
            AvpType::UTF8String(avp) => avp.serialize(),
            _ => Vec::new(),
        }
    }
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
