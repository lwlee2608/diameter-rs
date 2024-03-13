use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

use super::octetstring::OctetString;

pub enum AddressType {
    IPv4 = 1,
    IPv6 = 2,
}

impl AddressType {
    fn as_bytes(&self) -> [u8; 2] {
        match self {
            AddressType::IPv4 => [0, 1],
            AddressType::IPv6 => [0, 2],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Address(OctetString);

impl Address {
    pub fn new(value: Vec<u8>) -> Address {
        Address(OctetString::new(value))
    }

    pub fn from_ipv4_addr(addr: Ipv4Addr) -> Address {
        let octet = addr.octets();
        let addr_type = AddressType::IPv4;
        let mut value = Vec::with_capacity(6);
        value.extend_from_slice(&addr_type.as_bytes());
        value.extend_from_slice(&octet);
        Address(OctetString::new(value))
    }

    pub fn from_ipv6_addr(addr: Ipv6Addr) -> Address {
        let octet = addr.octets();
        let addr_type = AddressType::IPv6;
        let mut value = Vec::with_capacity(18);
        value.extend_from_slice(&addr_type.as_bytes());
        value.extend_from_slice(&octet);
        Address(OctetString::new(value))
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<Address> {
        let avp = OctetString::decode_from(reader, len)?;
        Ok(Address(avp))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.0.encode_to(writer)?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.length()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, &byte) in self.0.value().iter().enumerate() {
            if index > 0 {
                write!(f, " ")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_encode_decode_ipv4() {
        let ip_addr = Ipv4Addr::new(127, 0, 0, 1);
        let avp = Address::from_ipv4_addr(ip_addr);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Address::decode_from(&mut cursor, 6).unwrap();
        assert_eq!(avp.0.value(), vec![0, 1, 127, 0, 0, 1]);
    }

    #[test]

    fn test_encode_decode_ipv6() {
        let ip_addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        let avp = Address::from_ipv6_addr(ip_addr);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp_decoded = Address::decode_from(&mut cursor, encoded.len()).unwrap();
        assert_eq!(
            avp_decoded.0.value(),
            vec![0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
    }
}
