use crate::avp::AvpData;
use crate::error::Error;
use std::fmt;
use std::io::Read;

#[derive(Debug)]
pub struct OctetStringAvp(Vec<u8>);

impl OctetStringAvp {
    pub fn new(value: Vec<u8>) -> OctetStringAvp {
        OctetStringAvp(value)
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<OctetStringAvp, Error> {
        let mut b = vec![0u8; len];
        reader.read_exact(&mut b)?;
        Ok(OctetStringAvp(b))
    }
}

impl AvpData for OctetStringAvp {
    fn length(&self) -> u32 {
        self.0.len() as u32
    }

    fn serialize(&self) -> Vec<u8> {
        return self.0.clone();
    }
}

impl fmt::Display for OctetStringAvp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, &byte) in self.0.iter().enumerate() {
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
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_encode_decode_ascii() {
        let bytes = b"Hello World";
        let avp = OctetStringAvp::new(bytes.to_vec());
        let encoded = avp.serialize();
        let mut cursor = Cursor::new(&encoded);
        let avp = OctetStringAvp::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.0, bytes);
    }

    #[test]
    fn test_encode_decode_utf8() {
        let bytes = "世界,你好".as_bytes();
        let avp = OctetStringAvp::new(bytes.to_vec());
        let encoded = avp.serialize();
        let mut cursor = Cursor::new(&encoded);
        let avp = OctetStringAvp::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.0, bytes);
    }

    #[test]
    fn test_decode_invalid_utf8() {
        let bytes = vec![0x61, 0x62, 0x63, 0x64, 0x80];
        let avp = OctetStringAvp::new(bytes.to_vec());
        let encoded = avp.serialize();
        let mut cursor = Cursor::new(&encoded);
        let avp = OctetStringAvp::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.0, bytes);
    }
}