use crate::avp::AvpData;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct Integer32Avp(i32);

impl Integer32Avp {
    pub fn new(value: i32) -> Integer32Avp {
        Integer32Avp(value)
    }

    pub fn decode_from(b: &[u8]) -> Result<Integer32Avp, Box<dyn Error>> {
        if b.len() != 4 {
            return Err("Invalid Integer32Avp length".into());
        }

        let bytes: [u8; 4] = b.try_into()?;
        let num = i32::from_be_bytes(bytes);

        Ok(Integer32Avp(num))
    }
}

impl AvpData for Integer32Avp {
    fn serialize(&self) -> Vec<u8> {
        return self.0.to_be_bytes().to_vec();
    }
}

impl fmt::Display for Integer32Avp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let avp = Integer32Avp::new(1234567890);
        let bytes = avp.serialize();
        let avp = Integer32Avp::decode_from(&bytes).unwrap();
        assert_eq!(avp.0, 1234567890);
    }
}
