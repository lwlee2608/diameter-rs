use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::net::Ipv6Addr;

#[derive(Debug)]
pub struct IPv6Avp(Ipv6Addr);

impl IPv6Avp {
    pub fn new(value: Ipv6Addr) -> IPv6Avp {
        IPv6Avp(value)
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<IPv6Avp> {
        let mut b = [0; 16];
        reader.read_exact(&mut b)?;

        let ip = Ipv6Addr::new(
            (b[0] as u16) << 8 | b[1] as u16,
            (b[2] as u16) << 8 | b[3] as u16,
            (b[4] as u16) << 8 | b[5] as u16,
            (b[6] as u16) << 8 | b[7] as u16,
            (b[8] as u16) << 8 | b[9] as u16,
            (b[10] as u16) << 8 | b[11] as u16,
            (b[12] as u16) << 8 | b[13] as u16,
            (b[14] as u16) << 8 | b[15] as u16,
        );
        Ok(IPv6Avp(ip))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0.octets())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        16
    }
}

impl fmt::Display for IPv6Avp {
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
        let ipv6_addr = Ipv6Addr::new(
            0x664f, 0x0c54, 0x5729, 0xf308, 0x569f, 0xb682, 0x80b6, 0x0140,
        );
        let avp = IPv6Avp::new(ipv6_addr);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = IPv6Avp::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, ipv6_addr);
    }
}
