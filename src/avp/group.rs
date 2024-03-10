use crate::avp::Avp;
use crate::error::{Error, Result};
use std::io::Read;
use std::io::Seek;
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Grouped(Vec<Avp>);

impl Grouped {
    pub fn new(avps: Vec<Avp>) -> Grouped {
        Grouped(avps)
    }

    pub fn avps(&self) -> &[Avp] {
        &self.0
    }

    pub fn decode_from<R: Read + Seek>(reader: &mut R, len: usize) -> Result<Grouped> {
        let mut avps = Vec::new();

        let mut offset = 0;
        while offset < len {
            let avp = Avp::decode_from(reader)?;
            offset += avp.get_length() as usize;
            offset += avp.get_padding() as usize;
            avps.push(avp);
        }

        // sanity check, make sure everything is read
        if offset != len {
            return Err(Error::DecodeError(
                "invalid group avp, length mismatch".into(),
            ));
        }

        Ok(Grouped(avps))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        for avp in &self.0 {
            avp.encode_to(writer)?;
        }
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.0
            .iter()
            .map(|avp| avp.get_length() + avp.get_padding() as u32)
            .sum()
    }

    pub fn fmt(&self, f: &mut std::fmt::Formatter<'_>, depth: usize) -> std::fmt::Result {
        for avp in &self.0 {
            write!(f, "\n")?;
            avp.fmt(f, depth + 1)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Grouped {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(f, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avp;
    use crate::avp::enumerated::Enumerated;
    use crate::avp::unsigned32::Unsigned32;
    use crate::avp::AvpValue;

    #[test]
    fn test_encode_decode() {
        let avp = Grouped::new(vec![
            avp!(416, None, 0, Enumerated::new(1)),
            avp!(415, None, 0, Unsigned32::new(1000)),
        ]);
        assert_eq!(avp.avps().len(), 2);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = std::io::Cursor::new(&encoded);
        let avp = Grouped::decode_from(&mut cursor, encoded.len()).unwrap();
        assert_eq!(avp.avps().len(), 2);
        assert_eq!(avp.avps()[0].get_code(), 416);
        assert_eq!(avp.avps()[1].get_code(), 415);

        match avp.avps()[0].get_value() {
            AvpValue::Enumerated(v) => assert_eq!(v.value(), 1),
            _ => panic!("invalid value, expected Enumerated"),
        }
        match avp.avps()[1].get_value() {
            AvpValue::Unsigned32(v) => assert_eq!(v.value(), 1000),
            _ => panic!("invalid value, expected Unsigned32"),
        }
    }
}
