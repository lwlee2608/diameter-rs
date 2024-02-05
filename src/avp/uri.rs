use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

use crate::avp::OctetString;

#[derive(Debug, Clone)]
pub struct DiameterURI(OctetString);

impl DiameterURI {
    pub fn new(value: Vec<u8>) -> DiameterURI {
        DiameterURI(OctetString::new(value))
    }

    pub fn value(&self) -> &[u8] {
        self.0.value()
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<DiameterURI> {
        let avp = OctetString::decode_from(reader, len)?;
        Ok(DiameterURI(avp))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.0.encode_to(writer)?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.length()
    }
}

impl fmt::Display for DiameterURI {
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
