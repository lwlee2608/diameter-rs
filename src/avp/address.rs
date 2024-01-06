use crate::avp::AvpData;
use crate::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct AddressAvp<'a>(&'a [u8]);

impl<'a> AddressAvp<'a> {
    pub fn new(data: &[u8]) -> AddressAvp {
        AddressAvp(data)
    }

    pub fn decode_from(b: &'a [u8]) -> Result<AddressAvp, Error> {
        Ok(AddressAvp(b))
    }
}

impl AvpData for AddressAvp<'_> {
    fn length(&self) -> u32 {
        4
    }

    fn serialize(&self) -> Vec<u8> {
        return self.0.to_vec();
    }
}

impl fmt::Display for AddressAvp<'_> {
    // TODO
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AddressAvp")
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_encode_decode() {}
}
