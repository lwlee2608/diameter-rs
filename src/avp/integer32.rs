use crate::avp::AvpData;
use crate::error::Error;
use std::fmt;
use std::io::Read;

#[derive(Debug)]
pub struct Integer32Avp(i32);

impl Integer32Avp {
    pub fn new(value: i32) -> Integer32Avp {
        Integer32Avp(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Integer32Avp, Error> {
        let mut b = [0; 4];
        reader.read_exact(&mut b)?;
        let num = i32::from_be_bytes(b);
        Ok(Integer32Avp(num))
    }
}

impl AvpData for Integer32Avp {
    fn length(&self) -> u32 {
        4
    }

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
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_encode_decode() {
        let avp = Integer32Avp::new(1234567890);
        let encoded = avp.serialize();
        let mut cursor = Cursor::new(&encoded);
        let avp = Integer32Avp::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, 1234567890);
    }
}
