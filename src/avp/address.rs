use num_traits::ToPrimitive;

use crate::error::Error;
use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

#[derive(Debug, Clone)]
pub enum Value {
    IPv4(Ipv4Addr),
    IPv6(Ipv6Addr),
    E164(String),
}

#[derive(Debug, Clone)]
pub struct Address(Value);

impl Address {
    pub fn new(value: Value) -> Address {
        Address(value)
    }

    pub fn from_ipv4(ip: Ipv4Addr) -> Address {
        Address(Value::IPv4(ip))
    }

    pub fn from_ipv6(ip: Ipv6Addr) -> Address {
        Address(Value::IPv6(ip))
    }

    pub fn from_e164(str: String) -> Address {
        Address(Value::E164(str))
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<Address> {
        let mut b = [0; 2];
        reader.read_exact(&mut b)?;
        let avp = match b {
            [0, 1] => {
                if len != 6 {
                    return Err(Error::DecodeError("Invalid address length".into()));
                }
                let mut b = [0; 4];
                reader.read_exact(&mut b)?;
                let ip = Ipv4Addr::new(b[0], b[1], b[2], b[3]);
                Address(Value::IPv4(ip))
            }
            [0, 2] => {
                if len != 18 {
                    return Err(Error::DecodeError("Invalid address length".into()));
                }
                let mut b = [0; 16];
                reader.read_exact(&mut b)?;
                let ip = Ipv6Addr::new(
                    u16::from_be_bytes([b[0], b[1]]),
                    u16::from_be_bytes([b[2], b[3]]),
                    u16::from_be_bytes([b[4], b[5]]),
                    u16::from_be_bytes([b[6], b[7]]),
                    u16::from_be_bytes([b[8], b[9]]),
                    u16::from_be_bytes([b[10], b[11]]),
                    u16::from_be_bytes([b[12], b[13]]),
                    u16::from_be_bytes([b[14], b[15]]),
                );
                Address(Value::IPv6(ip))
            }
            [0, 8] => {
                if len > 17 {
                    return Err(Error::DecodeError(
                        "E164 address should not exceed max length of 15".into(),
                    ));
                }
                let mut b = [0; 15];
                let actual_len: usize = len - 2;
                let b = &mut b[0..actual_len];
                reader.read_exact(b)?;
                let s = String::from_utf8(b.to_vec())
                    .map_err(|e| Error::DecodeError(format!("invalid UTF8String: {}", e)))?;

                Address(Value::E164(s))
            }
            _ => return Err(Error::DecodeError("Unsupported address type".into())),
        };
        Ok(avp)
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        match &self.0 {
            Value::IPv4(ip) => {
                writer.write_all(&[0, 1])?;
                writer.write_all(&ip.octets())?;
            }
            Value::IPv6(ip) => {
                writer.write_all(&[0, 2])?;
                writer.write_all(&ip.octets())?;
            }
            Value::E164(str) => {
                writer.write_all(&[0, 8])?;
                writer.write_all(&str.as_bytes())?;
            }
        };
        Ok(())
    }

    pub fn length(&self) -> u32 {
        match &self.0 {
            Value::IPv4(_) => 6,
            Value::IPv6(_) => 18,
            Value::E164(utf8string) => utf8string.len().to_u32().unwrap(),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::IPv4(ip) => write!(f, "{}", ip),
            Value::IPv6(ip) => write!(f, "{}", ip),
            Value::E164(str) => write!(f, "{}", str),
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_encode_decode_ipv4() {
        let addr = Ipv4Addr::new(127, 0, 0, 1);
        let avp = Address::new(Value::IPv4(addr));
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Address::decode_from(&mut cursor, 6).unwrap();
        assert_eq!(avp.0.to_string(), "127.0.0.1");
    }

    #[test]
    fn test_encode_decode_ipv6() {
        let addr = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        let avp = Address::new(Value::IPv6(addr));
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Address::decode_from(&mut cursor, 18).unwrap();
        assert_eq!(avp.0.to_string(), "::1");
    }

    #[test]
    fn test_encode_decode_e164() {
        let avp = Address::new(Value::E164("359898000135".to_string()));
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Address::decode_from(&mut cursor, 14).unwrap();
        assert_eq!(avp.0.to_string(), "359898000135");
    }
}
