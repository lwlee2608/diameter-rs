use crate::error::{Error, Result};
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct UTF8String(String);

impl UTF8String {
    pub fn new(value: &str) -> UTF8String {
        UTF8String(value.to_string())
    }

    pub fn value(&self) -> &str {
        &self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<UTF8String> {
        let mut b = vec![0u8; len];
        reader.read_exact(&mut b)?;

        let s = String::from_utf8(b.to_vec())
            .map_err(|e| Error::DecodeError(format!("invalid UTF8String: {}", e)))?;
        Ok(UTF8String(s))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(self.0.as_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.len() as u32
    }
}

impl fmt::Display for UTF8String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_encode_decode_ascii() {
        let str = "Hello World";
        let avp = UTF8String::new(str);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = UTF8String::decode_from(&mut cursor, str.len()).unwrap();
        assert_eq!(avp.value(), str);
    }

    #[test]
    fn test_encode_decode_utf8() {
        let str = "世界,你好";
        let avp = UTF8String::new(str);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = UTF8String::decode_from(&mut cursor, str.len()).unwrap();
        assert_eq!(avp.value(), str);
    }

    #[test]
    fn test_decode_invalid_utf8() {
        let bytes = vec![0x61, 0x62, 0x63, 0x64, 0x80];
        let mut cursor = Cursor::new(&bytes);
        match UTF8String::decode_from(&mut cursor, 5) {
            Err(Error::DecodeError(msg)) => {
                assert_eq!(
                    msg,
                    "invalid UTF8String: invalid utf-8 sequence of 1 bytes from index 4"
                );
            }
            Err(_) => panic!("Expected a DecodeError, but got a different error"),
            Ok(_) => panic!("Expected an error, but got Ok"),
        }
    }
}
