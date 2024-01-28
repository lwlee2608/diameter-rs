use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct OctetString(Vec<u8>);

impl OctetString {
    pub fn new(value: Vec<u8>) -> OctetString {
        OctetString(value)
    }

    pub fn value(&self) -> &[u8] {
        &self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<OctetString> {
        let mut b = vec![0u8; len];
        reader.read_exact(&mut b)?;
        Ok(OctetString(b))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0)?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.len() as u32
    }
}

impl fmt::Display for OctetString {
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
        let avp = OctetString::new(bytes.to_vec());
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = OctetString::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.value(), bytes);
    }

    #[test]
    fn test_encode_decode_utf8() {
        let bytes = "世界,你好".as_bytes();
        let avp = OctetString::new(bytes.to_vec());
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = OctetString::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.value(), bytes);
    }

    #[test]
    fn test_decode_invalid_utf8() {
        let bytes = vec![0x61, 0x62, 0x63, 0x64, 0x80];
        let avp = OctetString::new(bytes.to_vec());
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = OctetString::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.value(), bytes);
    }
}
