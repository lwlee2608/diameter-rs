use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct Enumerated(i32);

impl Enumerated {
    pub fn new(value: i32) -> Enumerated {
        Enumerated(value)
    }

    pub fn value(&self) -> i32 {
        self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Enumerated> {
        let mut b = [0; 4];
        reader.read_exact(&mut b)?;
        let num = i32::from_be_bytes(b);
        Ok(Enumerated(num))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        4
    }
}

impl fmt::Display for Enumerated {
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
        let avp = Enumerated::new(-1234567890);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Enumerated::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, -1234567890);
    }
}
