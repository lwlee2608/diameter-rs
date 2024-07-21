use crate::avp::Avp;
use crate::dictionary::{self, Dictionary};
use crate::error::{Error, Result};
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::sync::Arc;

use super::AvpValue;

#[derive(Debug, Clone)]
pub struct Grouped {
    avps: Vec<Avp>,
    dict: Arc<Dictionary>,
}

impl Grouped {
    pub fn new(avps: Vec<Avp>, dict: Arc<Dictionary>) -> Grouped {
        Grouped { avps, dict }
    }

    pub fn avps(&self) -> &[Avp] {
        &self.avps
    }

    pub fn add(&mut self, avp: Avp) {
        self.avps.push(avp);
    }

    pub fn add_avp(&mut self, code: u32, vendor_id: Option<u32>, flags: u8, value: AvpValue) {
        let avp = Avp::new(code, vendor_id, flags, value, Arc::clone(&self.dict));
        self.add(avp);
    }

    pub fn decode_from<R: Read + Seek>(
        reader: &mut R,
        len: usize,
        dict: Arc<Dictionary>,
    ) -> Result<Grouped> {
        let mut avps = Vec::new();

        let mut offset = 0;
        while offset < len {
            let avp = Avp::decode_from(reader, Arc::clone(&dict))?;
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

        Ok(Grouped { avps, dict })
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        for avp in &self.avps {
            avp.encode_to(writer)?;
        }
        Ok(())
    }

    pub fn length(&self) -> u32 {
        self.avps
            .iter()
            .map(|avp| avp.get_length() + avp.get_padding() as u32)
            .sum()
    }

    pub fn fmt(&self, f: &mut std::fmt::Formatter<'_>, depth: usize) -> std::fmt::Result {
        let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
        for avp in &self.avps {
            write!(f, "\n")?;
            avp.fmt(f, depth + 1, &dict)?;
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
    use crate::avp::enumerated::Enumerated;
    use crate::avp::unsigned32::Unsigned32;
    use crate::avp::AvpValue;
    use crate::{avp, dictionary};
    use std::sync::Arc;

    #[test]
    fn test_new_grouped_avp() {
        let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
        let dict = Arc::new(dict);

        let mut grouped_avp = Grouped::new(vec![], Arc::clone(&dict));
        grouped_avp.add_avp(416, None, 0, Enumerated::new(1).into());
        grouped_avp.add_avp(415, None, 0, Unsigned32::new(1000).into());

        assert_eq!(grouped_avp.avps().len(), 2);
        assert_eq!(grouped_avp.avps()[0].get_code(), 416);
        assert_eq!(grouped_avp.avps()[1].get_code(), 415);
    }

    #[test]
    fn test_encode_decode() {
        let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
        let dict = Arc::new(dict);

        let avp = Grouped::new(
            vec![
                avp!(416, None, 0, Enumerated::new(1), Arc::clone(&dict)),
                avp!(415, None, 0, Unsigned32::new(1000), Arc::clone(&dict)),
            ],
            Arc::clone(&dict),
        );
        assert_eq!(avp.avps().len(), 2);
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = std::io::Cursor::new(&encoded);
        let avp = Grouped::decode_from(&mut cursor, encoded.len(), dict).unwrap();
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
