use crate::error::Error;
use std::fmt;
use std::io::Read;
use std::io::Write;

use super::octetstring::OctetStringAvp;

#[derive(Debug)]
pub struct AddressAvp(OctetStringAvp);

impl AddressAvp {
    pub fn new(value: Vec<u8>) -> AddressAvp {
        AddressAvp(OctetStringAvp::new(value))
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<AddressAvp, Error> {
        let avp = OctetStringAvp::decode_from(reader, len)?;
        Ok(AddressAvp(avp))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.0.encode_to(writer)?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.length()
    }
}

impl fmt::Display for AddressAvp {
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
