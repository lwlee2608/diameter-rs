use crate::avp::AvpDataType;
use std::error::Error;

#[derive(Debug)]
pub struct UTF8StringAvp(String);

impl UTF8StringAvp {
    pub fn new(value: String) -> UTF8StringAvp {
        UTF8StringAvp(value)
    }
    pub fn decode_from(b: &[u8]) -> Result<UTF8StringAvp, Box<dyn Error>> {
        let s = String::from_utf8(b.to_vec())?;
        Ok(UTF8StringAvp(s))
    }
}

impl AvpDataType for UTF8StringAvp {
    fn serialize(&self) -> Vec<u8> {
        return self.0.as_bytes().to_vec();
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
}
