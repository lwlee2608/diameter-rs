use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Unsigned32(u32);

impl Unsigned32 {
    pub fn new(value: u32) -> Unsigned32 {
        Unsigned32(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Unsigned32> {
        let mut b = [0; 4];
        reader.read_exact(&mut b)?;
        let num = u32::from_be_bytes(b);
        Ok(Unsigned32(num))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        4
    }
}

impl fmt::Display for Unsigned32 {
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
        let avp = Unsigned32::new(1234567890);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Unsigned32::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, 1234567890);
    }
}
