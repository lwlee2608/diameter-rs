use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Float32(f32);

impl Float32 {
    pub fn new(value: f32) -> Float32 {
        Float32(value)
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Float32> {
        let mut b = [0; 4];
        reader.read_exact(&mut b)?;
        let num = f32::from_be_bytes(b);
        Ok(Float32(num))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        4
    }
}

impl fmt::Display for Float32 {
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
        let avp = Float32::new(-3.142);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Float32::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.0, -3.142);
    }
}
