use crate::avp::OctetStringAvp;
use crate::error::Error;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct DiameterIdentity(OctetStringAvp);

impl DiameterIdentity {
    pub fn new(value: Vec<u8>) -> DiameterIdentity {
        DiameterIdentity(OctetStringAvp::new(value))
    }

    pub fn value(&self) -> &[u8] {
        self.0.value()
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<DiameterIdentity, Error> {
        let avp = OctetStringAvp::decode_from(reader, len)?;
        Ok(DiameterIdentity(avp))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.0.encode_to(writer)?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.length()
    }
}

impl fmt::Display for DiameterIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, &byte) in self.0.value().iter().enumerate() {
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
        let bytes = b"example.com";
        let avp = DiameterIdentity::new(bytes.to_vec());
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = OctetStringAvp::decode_from(&mut cursor, bytes.len()).unwrap();
        assert_eq!(avp.value(), bytes);
    }
}
