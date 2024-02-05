use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Unsigned64(u64);

impl Unsigned64 {
    pub fn new(value: u64) -> Unsigned64 {
        Unsigned64(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Unsigned64> {
        let mut b = [0; 8];
        reader.read_exact(&mut b)?;
        let num = u64::from_be_bytes(b);
        Ok(Unsigned64(num))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        8
    }
}

impl fmt::Display for Unsigned64 {
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
        let avp = Unsigned64::new(123456789000000000);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Unsigned64::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, 123456789000000000);
    }
}
