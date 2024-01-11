use crate::avp::Avp;
use crate::dictionary;
use crate::error::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt;
use std::io::Read;
use std::io::Seek;
use std::io::Write;

/*
 * Raw packet format:
 *   0                   1                   2                   3
 *   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |    Version    |                 Message Length                |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  | command flags |                  Command-Code                 |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                         Application-ID                        |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                      Hop-by-Hop Identifier                    |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                      End-to-End Identifier                    |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                              AVPs                             |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                              ...                              |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *
 * Command Flags:
 *   0 1 2 3 4 5 6 7
 *  +-+-+-+-+-+-+-+-+  R(equest), P(roxyable), E(rror)
 *  |R P E T r r r r|  T(Potentially re-transmitted message), r(eserved)
 *  +-+-+-+-+-+-+-+-+
 *
 */

pub const HEADER_LENGTH: u32 = 20;
pub const REQUEST_FLAG: u8 = 0x80;
pub const PROXYABLE_FLAG: u8 = 0x40;
pub const ERROR_FLAG: u8 = 0x20;
pub const RETRANSMIT_FLAG: u8 = 0x10;

#[derive(Debug)]
pub struct DiameterMessage {
    header: DiameterHeader,
    avps: Vec<Avp>,
}

#[derive(Debug)]
pub struct DiameterHeader {
    version: u8,
    length: u32,
    flags: u8,
    code: CommandCode,
    application_id: ApplicationId,
    hop_by_hop_id: u32,
    end_to_end_id: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
pub enum CommandCode {
    Error = 0,
    CapabilitiesExchange = 257,
    DeviceWatchdog = 280,
    DisconnectPeer = 282,
    ReAuth = 258,
    SessionTerminate = 275,
    AbortSession = 274,
    CreditControl = 272,
    SpendingLimit = 8388635,
    SpendingStatusNotification = 8388636,
    Accounting = 271,
    AA = 265,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, FromPrimitive)]
pub enum ApplicationId {
    Common = 0,
    Accounting = 3,
    CreditControl = 4,
    Gx = 16777238,
    Rx = 16777236,
    Sy = 16777302,
}

impl DiameterMessage {
    pub fn new(
        code: CommandCode,
        application_id: ApplicationId,
        flags: u8,
        hop_by_hop_id: u32,
        end_to_end_id: u32,
    ) -> DiameterMessage {
        let header = DiameterHeader {
            version: 1,
            length: HEADER_LENGTH,
            flags,
            code,
            application_id,
            hop_by_hop_id,
            end_to_end_id,
        };
        let avps = Vec::new();
        DiameterMessage { header, avps }
    }

    pub fn get_avps(&self) -> &Vec<Avp> {
        &self.avps
    }

    pub fn add_avp(&mut self, avp: Avp) {
        self.header.length += avp.get_length() + avp.get_padding() as u32;
        self.avps.push(avp);
    }

    pub fn get_length(&self) -> u32 {
        self.header.length
    }

    // pub fn decode_from<'a>(b: &'a [u8]) -> Result<DiameterMessage, Box<dyn Error>> {
    pub fn decode_from<R: Read + Seek>(reader: &mut R) -> Result<DiameterMessage, Error> {
        let header = DiameterHeader::decode_from(reader)?;
        let mut avps = Vec::new();

        let total_length = header.length;
        let mut offset = HEADER_LENGTH;
        while offset < total_length {
            let avp = Avp::decode_from(reader)?;
            offset += avp.get_length();
            offset += avp.get_padding() as u32;
            avps.push(avp);
        }

        // sanity check, make sure everything is read
        if offset != total_length {
            return Err(Error::DecodeError(
                "invalid diameter message, length mismatch".into(),
            ));
        }

        Ok(DiameterMessage { header, avps })
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.header.encode_to(writer)?;

        for avp in &self.avps {
            avp.encode_to(writer)?;
        }

        Ok(())
    }
}

impl DiameterHeader {
    // pub fn decode_from<'a>(b: &'a [u8]) -> Result<DiameterHeader, Box<dyn Error>> {
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<DiameterHeader, Error> {
        let mut b = [0; HEADER_LENGTH as usize];
        reader.read_exact(&mut b)?;

        if b.len() < HEADER_LENGTH as usize {
            return Err(Error::DecodeError(
                "invalid diameter header, too short".into(),
            ));
        }

        let version = b[0];
        let length = u32::from_be_bytes([0, b[1], b[2], b[3]]);
        let flags = b[4];

        let code = u32::from_be_bytes([0, b[5], b[6], b[7]]);
        let application_id = u32::from_be_bytes([b[8], b[9], b[10], b[11]]);
        let hop_by_hop_id = u32::from_be_bytes([b[12], b[13], b[14], b[15]]);
        let end_to_end_id = u32::from_be_bytes([b[16], b[17], b[18], b[19]]);

        let code = CommandCode::from_u32(code)
            .ok_or_else(|| Error::DecodeError(format!("unknown command code: {}", code).into()))?;

        let application_id = ApplicationId::from_u32(application_id).ok_or_else(|| {
            Error::DecodeError(format!("unknown application id: {}", application_id).into())
        })?;

        Ok(DiameterHeader {
            version,
            length,
            flags,
            code,
            application_id,
            hop_by_hop_id,
            end_to_end_id,
        })
    }

    pub fn encode_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        // version
        writer.write_all(&[self.version])?;

        // Length
        let length_bytes = &self.length.to_be_bytes()[1..4];
        writer.write_all(length_bytes)?;

        // flags
        writer.write_all(&[self.flags])?;

        // Code
        let code = self.code as u32;
        let code_bytes = &code.to_be_bytes()[1..4];
        writer.write_all(code_bytes)?;

        // Application-ID
        let application_id = self.application_id as u32;
        writer.write_all(&application_id.to_be_bytes())?;

        // Hop-by-Hop Identifier and End-to-End Identifier
        writer.write_all(&self.hop_by_hop_id.to_be_bytes())?;
        writer.write_all(&self.end_to_end_id.to_be_bytes())?;

        Ok(())
    }
}

impl CommandCode {
    pub fn from_u32(code: u32) -> Option<CommandCode> {
        FromPrimitive::from_u32(code)
    }
}

impl ApplicationId {
    pub fn from_u32(application_id: u32) -> Option<ApplicationId> {
        FromPrimitive::from_u32(application_id)
    }
}

impl fmt::Display for DiameterMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.header)?;
        write!(
            f,
            "  {:<40} {:>8} {:>5}  {} {} {}  {:<16}  {}\n",
            "AVP", "Vendor", "Code", "V", "M", "P", "Type", "Value"
        )?;

        for avp in &self.avps {
            write!(f, "{}\n", avp)?;
        }

        Ok(())
    }
}

impl fmt::Display for DiameterHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let request_flag = if self.flags & REQUEST_FLAG != 0 {
            "Request"
        } else {
            "Answer"
        };
        let error_flag = if self.flags & ERROR_FLAG != 0 {
            "Error"
        } else {
            ""
        };
        let proxyable_flag = if self.flags & PROXYABLE_FLAG != 0 {
            "Proxyable"
        } else {
            ""
        };
        let retransmit_flag = if self.flags & RETRANSMIT_FLAG != 0 {
            "Retransmit"
        } else {
            ""
        };

        write!(
            f,
            "{} {}({}) {}({}) {}{}{}{} {}, {}",
            self.version,
            self.code,
            self.code as u32,
            self.application_id,
            self.application_id as u32,
            request_flag,
            error_flag,
            proxyable_flag,
            retransmit_flag,
            self.hop_by_hop_id,
            self.end_to_end_id
        )
    }
}

impl fmt::Display for CommandCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for ApplicationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Avp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let avp_name = dictionary::DEFAULT_DICT
            .get_avp_name(self.get_code() as u32)
            .unwrap_or("Unknown");

        let vendor_id = match self.get_vendor_id() {
            Some(v) => v.to_string(),
            None => "".to_string(),
        };

        write!(
            f,
            "  {:<40} {:>8} {:>5}  {} {} {}  {:<16}  {}",
            avp_name,
            vendor_id,
            self.get_code(),
            get_bool_unicode(self.get_flags().vendor),
            get_bool_unicode(self.get_flags().mandatory),
            get_bool_unicode(self.get_flags().private),
            self.get_value().get_type_name(),
            self.get_value().to_string()
        )
    }
}

fn get_bool_unicode(v: bool) -> &'static str {
    if v {
        "✓"
    } else {
        "✗"
    }
}

#[cfg(test)]
mod tests {
    use crate::avp;
    use crate::avp::enumerated::EnumeratedAvp;
    use crate::avp::group::GroupAvp;
    use crate::avp::identity::IdentityAvp;
    use crate::avp::unsigned32::Unsigned32Avp;
    use crate::avp::utf8string::UTF8StringAvp;
    use crate::avp::AvpValue;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_decode_encode_header() {
        let data = [
            0x01, 0x00, 0x00, 0x14, // version, length
            0x80, 0x00, 0x01, 0x10, // flags, code
            0x00, 0x00, 0x00, 0x04, // application_id
            0x00, 0x00, 0x00, 0x03, // hop_by_hop_id
            0x00, 0x00, 0x00, 0x04, // end_to_end_id
        ];

        let mut cursor = Cursor::new(&data);
        let header = DiameterHeader::decode_from(&mut cursor).unwrap();
        // let header = DiameterHeader::decode_from(&data).unwrap();

        assert_eq!(header.version, 1);
        assert_eq!(header.length, 20);
        assert_eq!(header.flags, REQUEST_FLAG);
        assert_eq!(header.code, CommandCode::CreditControl);
        assert_eq!(header.application_id, ApplicationId::CreditControl);
        assert_eq!(header.hop_by_hop_id, 3);
        assert_eq!(header.end_to_end_id, 4);

        let mut encoded = Vec::new();
        header.encode_to(&mut encoded).unwrap();
        assert_eq!(encoded, data);
    }

    #[test]
    fn test_decode_encode_diameter_message() {
        let data = [
            0x01, 0x00, 0x00, 0x34, // version, length
            0x80, 0x00, 0x01, 0x10, // flags, code
            0x00, 0x00, 0x00, 0x04, // application_id
            0x00, 0x00, 0x00, 0x03, // hop_by_hop_id
            0x00, 0x00, 0x00, 0x04, // end_to_end_id
            0x00, 0x00, 0x02, 0x3B, // avp code
            0x40, 0x00, 0x00, 0x0C, // flags, length
            0x00, 0x00, 0x04, 0xB0, // value
            0x00, 0x00, 0x00, 0x1E, // avp code
            0x00, 0x00, 0x00, 0x12, // flags, length
            0x66, 0x6F, 0x6F, 0x62, // value
            0x61, 0x72, 0x31, 0x32, // value
            0x33, 0x34, 0x00, 0x00,
        ];

        let mut cursor = Cursor::new(&data);
        let message = DiameterMessage::decode_from(&mut cursor).unwrap();
        println!("diameter message: {}", message);

        let avps = &message.avps;
        assert_eq!(avps.len(), 2);
        let avp0 = &avps[0];
        assert_eq!(avp0.get_code(), 571);
        assert_eq!(avp0.get_length(), 12);
        assert_eq!(avp0.get_flags().vendor, false);
        assert_eq!(avp0.get_flags().mandatory, true);
        assert_eq!(avp0.get_flags().private, false);
        assert_eq!(avp0.get_vendor_id(), None);
        match avp0.get_value() {
            AvpValue::Integer32(ref v) => assert_eq!(v.value(), 1200),
            _ => panic!("unexpected avp type"),
        }
        let avp1 = &avps[1];
        assert_eq!(avp1.get_code(), 30);
        assert_eq!(avp1.get_length(), 18);
        assert_eq!(avp1.get_flags().vendor, false);
        assert_eq!(avp1.get_flags().mandatory, false);
        assert_eq!(avp1.get_flags().private, false);
        assert_eq!(avp1.get_vendor_id(), None);
        match avp1.get_value() {
            AvpValue::UTF8String(ref v) => assert_eq!(v.value(), "foobar1234"),
            _ => panic!("unexpected avp type"),
        }

        let mut encoded = Vec::new();
        message.encode_to(&mut encoded).unwrap();
        assert_eq!(encoded, data);
    }

    #[test]
    fn test_diameter_struct() {
        let mut message = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            REQUEST_FLAG | PROXYABLE_FLAG,
            1123158610,
            3102381851,
        );

        message.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
        message.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
        message.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345888"), true));
        message.add_avp(avp!(268, None, Unsigned32Avp::new(2001), true));
        message.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
        message.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
        message.add_avp(avp!(
            873,
            Some(10415),
            GroupAvp::new(vec![avp!(
                874,
                Some(10415),
                GroupAvp::new(vec![avp!(30, None, UTF8StringAvp::new("10999"), true)]),
                true
            )]),
            true
        ));

        // encode
        let mut encoded = Vec::new();
        message.encode_to(&mut encoded).unwrap();

        // decode
        let mut cursor = Cursor::new(&encoded);
        let message = DiameterMessage::decode_from(&mut cursor).unwrap();

        println!("decoded message:\n{}", message);
    }

    #[test]
    fn test_decode_ccr() {
        let data = [
            0x01, 0x00, 0x00, 0x54, 0x00, 0x00, 0x01, 0x10, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x08, 0x40, 0x00, 0x00, 0x0E,
            0x73, 0x65, 0x72, 0x76, 0x65, 0x72, 0x00, 0x00, 0x00, 0x00, 0x01, 0x28, 0x40, 0x00,
            0x00, 0x13, 0x73, 0x65, 0x72, 0x76, 0x65, 0x72, 0x52, 0x65, 0x61, 0x6C, 0x6D, 0x00,
            0x00, 0x00, 0x01, 0x0C, 0x40, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x07, 0xD1, 0x00, 0x00,
            0x01, 0x07, 0x40, 0x00, 0x00, 0x0F, 0x73, 0x65, 0x73, 0x3B, 0x31, 0x32, 0x33, 0x00,
        ];

        let mut cursor = Cursor::new(&data);
        let message = DiameterMessage::decode_from(&mut cursor).unwrap();
        println!("diameter message: {}", message);
    }
}
