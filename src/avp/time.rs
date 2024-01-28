use crate::error::{Error, Result};
use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use std::fmt;
use std::io::Read;
use std::io::Write;

const RFC868_OFFSET: u32 = 2208988800; // Diff. between 1970 and 1900 in seconds.

#[derive(Debug)]
pub struct TimeAvp(DateTime<Utc>);

impl TimeAvp {
    pub fn new(time: DateTime<Utc>) -> Self {
        TimeAvp(time)
    }

    pub fn value(&self) -> &DateTime<Utc> {
        &self.0
    }

    pub fn decode_from<R: Read>(reader: &mut R) -> Result<TimeAvp> {
        let mut b = [0; 4];
        reader.read_exact(&mut b)?;

        let diameter_timestamp = u32::from_be_bytes(b); // seconds since 1900
        let unix_timestamp = diameter_timestamp as i64 - RFC868_OFFSET as i64;
        let timestamp = Utc
            .timestamp_opt(unix_timestamp, 0)
            .single()
            .ok_or_else(|| Error::DecodeError("Invalid time".to_string()))?;

        Ok(TimeAvp(timestamp))
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let unix_timestamp = self.0.timestamp();
        let diameter_timestamp = unix_timestamp + RFC868_OFFSET as i64;
        if diameter_timestamp > u32::MAX as i64 {
            Err(Error::EncodeError(
                "Time is too far in the future to fit into 32 bits".to_string(),
            ))?;
        }
        let diameter_timestamp = diameter_timestamp as u32;
        writer.write_all(&diameter_timestamp.to_be_bytes())?;
        Ok(())
    }

    pub fn length(&self) -> u32 {
        4
    }
}

impl fmt::Display for TimeAvp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::io::Cursor;

    #[test]
    fn test_encode_decode() {
        let now: DateTime<Utc> = Utc.with_ymd_and_hms(2024, 1, 10, 10, 35, 58).unwrap();
        let avp = TimeAvp::new(now.into());
        let mut encoded = Vec::new();
        avp.encode_to(&mut encoded).unwrap();
        let mut cursor = Cursor::new(&encoded);
        let avp = TimeAvp::decode_from(&mut cursor).unwrap();
        assert_eq!(*avp.value(), now);
    }
}
