use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct IPv4Avp(Ipv4Addr);

impl IPv4Avp {
    pub fn new(value: Ipv4Addr) -> IPv4Avp {
        IPv4Avp(value)
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<IPv4Avp> {
        let mut b = [0; 4];
        reader.read_exact(&mut b)?;

        let ip = Ipv4Addr::new(b[0], b[1], b[2], b[3]);
        Ok(IPv4Avp(ip))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0.octets())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        4
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
    use std::io::Cursor;

    #[test]
    fn test_encode_decode() {
        let avp = IPv4Avp::new(Ipv4Addr::new(127, 0, 0, 1));
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = IPv4Avp::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, Ipv4Addr::new(127, 0, 0, 1));
    }
}
