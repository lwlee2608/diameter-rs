use crate::avp::UTF8StringAvp;
use crate::error::Error;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct IdentityAvp(UTF8StringAvp);

impl IdentityAvp {
    pub fn new(value: &str) -> IdentityAvp {
        IdentityAvp(UTF8StringAvp::new(value))
    }

    pub fn value(&self) -> &str {
        self.0.value()
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<IdentityAvp, Error> {
        let avp = UTF8StringAvp::decode_from(reader, len)?;
        Ok(IdentityAvp(avp))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.0.encode_to(writer)?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.length()
    }
}

impl fmt::Display for IdentityAvp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.value())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_encode_decode_ascii() {
        let bytes = "example.com";
        let avp = IdentityAvp::new(bytes);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = UTF8StringAvp::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.value(), bytes);
    }
}
