use crate::error::Result;
use std::fmt;
use std::io::Read;
use std::io::Write;

#[derive(Debug)]
pub struct Float64Avp(f64);

impl Float64Avp {
    pub fn new(value: f64) -> Float64Avp {
        Float64Avp(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<Float64Avp> {
        let mut b = [0; 8];
        reader.read_exact(&mut b)?;
        let num = f64::from_be_bytes(b);
        Ok(Float64Avp(num))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.0.to_be_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        8
    }
}

impl fmt::Display for Float64Avp {
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
        let avp = Float64Avp::new(-3.142);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = Float64Avp::decode_from(&mut cursor).unwrap();
        assert_eq!(avp.value(), -3.142);
    }
}
