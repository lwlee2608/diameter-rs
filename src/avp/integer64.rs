use crate::error::Error;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct Integer64Avp(i64);

impl Integer64Avp {
    pub fn new(value: i64) -> Integer64Avp {
        Integer64Avp(value)
    }

    pub fn value(&self) -> i64 {
        self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Integer64Avp, Error> {
        let mut b = [0; 8];
        reader.read_exact(&mut b)?;
        let num = i64::from_be_bytes(b);
        Ok(Integer64Avp(num))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        8
    }
}

impl fmt::Display for Integer64Avp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_encode_decode() {
        let avp = Integer64Avp::new(-123456789000000000);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Integer64Avp::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, -123456789000000000);
    }
}
