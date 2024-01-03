use crate::avp::AvpDataType;
use std::error::Error;

#[derive(Debug)]
pub struct AddressAvp<'a>(&'a [u8]);

impl<'a> AddressAvp<'a> {
    pub fn new(data: &[u8]) -> AddressAvp {
        AddressAvp(data)
    }

    pub fn decode_from(b: &'a [u8]) -> Result<AddressAvp, Box<dyn Error>> {
        Ok(AddressAvp(b))
    }
}

impl AvpDataType for AddressAvp<'_> {
    fn serialize(&self) -> Vec<u8> {
        return self.0.to_vec();
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_encode_decode() {}
}
