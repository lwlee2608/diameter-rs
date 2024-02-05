use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

use super::octetstring::OctetString;

#[derive(Debug, Clone)]
pub struct Address(OctetString);

impl Address {
    pub fn new(value: Vec<u8>) -> Address {
        Address(OctetString::new(value))
    }

    pub fn decode_from<R: Read>(reader: &mut R, len: usize) -> Result<Address> {
        let avp = OctetString::decode_from(reader, len)?;
        Ok(Address(avp))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.0.encode_to(writer)?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0.length()
    }
}

impl fmt::Display for Address {
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
