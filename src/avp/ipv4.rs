use crate::avp::AvpData;
use crate::error::Error;
use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct IPv4Avp(Ipv4Addr);

impl IPv4Avp {
    pub fn new(value: Ipv4Addr) -> IPv4Avp {
        IPv4Avp(value)
    }

    pub fn decode_from(b: &[u8]) -> Result<IPv4Avp, Error> {
        if b.len() != 4 {
            return Err(Error::DecodeError("Invalid IPv4 address length".into()));
        }

        let ip = Ipv4Addr::new(b[0], b[1], b[2], b[3]);
        Ok(IPv4Avp(ip))
    }
}

impl AvpData for IPv4Avp {
    fn serialize(&self) -> Vec<u8> {
        return self.0.octets().to_vec();
    }
}

impl fmt::Display for IPv4Avp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let avp = IPv4Avp::new(Ipv4Addr::new(127, 0, 0, 1));
        let bytes = avp.serialize();
        let avp = IPv4Avp::decode_from(&bytes).unwrap();
        assert_eq!(avp.0, Ipv4Addr::new(127, 0, 0, 1));
    }
}
