use crate::avp::AvpDataType;
use std::error::Error;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct IPv4Avp(Ipv4Addr);

impl Ipv4Addr {
    pub fn new(value: Ipv4Addr) -> IPv4Avp {
        IPv4Avp(value)
    }

    pub fn decode_from(b: &[u8]) -> Result<IPv4Avp, Box<dyn Error>> {
        if b.len() != 4 {
            return Err("Invalid IPv4 address length".into());
        }

        let ip = Ipv4Addr::from(b);
        Ok(IPv4Avp(ip))
    }
}

impl AvpDataType for IPv4Avp {
    fn serialize(&self) -> Vec<u8> {
        return self.0.octets().to_vec();
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
