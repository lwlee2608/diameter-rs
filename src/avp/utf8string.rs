use crate::avp::AvpData;
use crate::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct UTF8StringAvp(String);

impl UTF8StringAvp {
    pub fn new(value: String) -> UTF8StringAvp {
        UTF8StringAvp(value)
    }

    pub fn decode_from(b: &[u8]) -> Result<UTF8StringAvp, Error> {
        let s = String::from_utf8(b.to_vec())
            .map_err(|e| Error::DecodeError(format!("invalid UTF8StringAvp: {}", e)))?;

        Ok(UTF8StringAvp(s))
    }
}

impl AvpData for UTF8StringAvp {
    fn serialize(&self) -> Vec<u8> {
        return self.0.as_bytes().to_vec();
    }
}

impl fmt::Display for UTF8StringAvp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let avp = UTF8StringAvp::new("Hello World".to_string());
        let bytes = avp.serialize();
        let avp = UTF8StringAvp::decode_from(&bytes).unwrap();
        assert_eq!(avp.0, "Hello World".to_string());
    }

    #[test]
    fn test_encode_decode_utf8() {
        let avp = UTF8StringAvp::new("世界,你好".to_string());
        let bytes = avp.serialize();
        let avp = UTF8StringAvp::decode_from(&bytes).unwrap();
        assert_eq!(avp.0, "世界,你好".to_string());
    }

    #[test]
    fn test_decode_invalid_utf8() {
        let bytes = vec![0x61, 0x62, 0x63, 0x64, 0x80];
        match UTF8StringAvp::decode_from(&bytes) {
            Err(Error::DecodeError(msg)) => {
                assert_eq!(
                    msg,
                    "invalid UTF8StringAvp: invalid utf-8 sequence of 1 bytes from index 4"
                );
            }
            Err(_) => panic!("Expected a DecodeError, but got a different error"),
            Ok(_) => panic!("Expected an error, but got Ok"),
        }
    }
}
