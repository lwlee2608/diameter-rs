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
    pub type_: AvpType,
    pub value: Box<dyn AvpData>,
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

pub trait AvpData: std::fmt::Debug + std::fmt::Display {
    fn serialize(&self) -> Vec<u8>;
}
