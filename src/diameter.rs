/*
 * Diameter Header.
 *
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
 *
 * Command Flags:
 *   0 1 2 3 4 5 6 7
 *  +-+-+-+-+-+-+-+-+  R(equest), P(roxyable), E(rror)
 *  |R P E T r r r r|  T(Potentially re-transmitted message), r(eserved)
 *  +-+-+-+-+-+-+-+-+
 *
 */
use crate::avp::Avp;
use crate::error::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt;
use std::io::Read;
use std::io::Seek;

const HEADER_LENGTH: u32 = 20;

#[derive(Debug)]
pub struct DiameterMessage {
    header: DiameterHeader,
    avps: Vec<Avp>,
}

#[derive(Debug)]
pub struct DiameterHeader {
    version: u8,
    length: u32,
    flags: CommandFlags,
    code: CommandCode,
    application_id: ApplicationId,
    hop_by_hop_id: u32,
    end_to_end_id: u32,
}

#[derive(Debug)]
pub struct CommandFlags {
    pub request: bool,
    pub proxyable: bool,
    pub error: bool,
    pub retransmit: bool,
}

#[derive(Debug, Clone, PartialEq, FromPrimitive)]
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

#[derive(Debug, Clone, PartialEq, FromPrimitive)]
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
        flags: CommandFlags,
        hop_by_hop_id: u32,
        end_to_end_id: u32,
        avps: Vec<Avp>,
    ) -> DiameterMessage {
        let header = DiameterHeader {
            version: 1,
            length: 0,
            flags,
            code,
            application_id,
            hop_by_hop_id,
            end_to_end_id,
        };
        DiameterMessage { header, avps }
    }

    pub fn get_header(&self) -> &DiameterHeader {
        &self.header
    }

    pub fn get_avps(&self) -> &Vec<Avp> {
        &self.avps
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
            offset += avp.get_padding();
            avps.push(avp);
        }

        // sanity check, make sure everything is read
        if offset != total_length {
            return Err(Error::DecodeError(
                "Invalid diameter message, length mismatch".into(),
            ));
        }

        Ok(DiameterMessage { header, avps })
    }
}

#[rustfmt::skip]
impl DiameterHeader {
    // pub fn decode_from<'a>(b: &'a [u8]) -> Result<DiameterHeader, Box<dyn Error>> {
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<DiameterHeader, Error> {
        let mut b = [0; HEADER_LENGTH as usize];
        reader.read_exact(&mut b)?;

        if b.len() < HEADER_LENGTH as usize {
            return Err(Error::DecodeError("Invalid diameter header, too short".into()));
        }

        let version = b[0];
        let length = u32::from_be_bytes([0, b[1], b[2], b[3]]);
        let flags = CommandFlags {
            request:    (b[4] & 0x80) != 0,
            proxyable:  (b[4] & 0x40) != 0,
            error:      (b[4] & 0x20) != 0,
            retransmit: (b[4] & 0x10) != 0,
        };

        let code            = u32::from_be_bytes([0,     b[5],  b[6],  b[7]]);
        let application_id  = u32::from_be_bytes([b[8],  b[9],  b[10], b[11]]);
        let hop_by_hop_id   = u32::from_be_bytes([b[12], b[13], b[14], b[15]]);
        let end_to_end_id   = u32::from_be_bytes([b[16], b[17], b[18], b[19]]);

        Ok(DiameterHeader {
            version,
            length,
            flags,
            code:           FromPrimitive::from_u32(code).unwrap(),
            application_id: FromPrimitive::from_u32(application_id).unwrap(),
            hop_by_hop_id,
            end_to_end_id,
        })
    }
}

impl fmt::Display for DiameterMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.get_header())?;
        write!(
            f,
            "  {:<40} {:>8} {:>5}  {} {} {}  {:<16}  {}\n",
            "AVP", "Vendor", "Code", "V", "M", "P", "Type", "Value"
        )?;

        for avp in self.get_avps() {
            write!(f, "{}\n", avp)?;
        }

        Ok(())
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
        let avp_name = get_avp_name(self.get_code());
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
            self.get_value().get_type(),
            self.get_value().to_string()
        )
    }
}

impl fmt::Display for DiameterHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let request_flag = if self.flags.request {
            "Request"
        } else {
            "Answer"
        };
        let error_flag = if self.flags.error { "Error" } else { "" };
        let proxyable_flag = if self.flags.proxyable {
            "Proxyable"
        } else {
            ""
        };
        let retransmit_flag = if self.flags.retransmit {
            "Retransmit"
        } else {
            ""
        };

        write!(
            f,
            "{} {}({}) {}({}) {}{}{}{} {}, {}",
            self.version,
            self.code,
            self.code.clone() as u32,
            self.application_id,
            self.application_id.clone() as u32,
            request_flag,
            error_flag,
            proxyable_flag,
            retransmit_flag,
            self.hop_by_hop_id,
            self.end_to_end_id
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

fn get_avp_name(code: u32) -> String {
    match code {
        264 => "Session-Id".to_string(),
        296 => "Origin-Realm".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::avp::integer32::Integer32Avp;
    use crate::avp::utf8string::UTF8StringAvp;
    use crate::avp::{AvpFlags, AvpType};
    use crate::diameter::CommandFlags;

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_decode_header() {
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
        assert_eq!(header.flags.request, true);
        assert_eq!(header.flags.proxyable, false);
        assert_eq!(header.flags.error, false);
        assert_eq!(header.flags.retransmit, false);
        assert_eq!(header.code, CommandCode::CreditControl);
        assert_eq!(header.application_id, ApplicationId::CreditControl);
        assert_eq!(header.hop_by_hop_id, 3);
        assert_eq!(header.end_to_end_id, 4);
    }

    #[test]
    fn test_decode_diameter_message() {
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
            AvpType::Integer32(ref v) => assert_eq!(v.value(), 1200),
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
            AvpType::UTF8String(ref v) => assert_eq!(v.value(), "foobar1234"),
            _ => panic!("unexpected avp type"),
        }
    }

    #[test]
    fn test_diameter_struct() {
        let message = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            CommandFlags {
                request: true,
                proxyable: false,
                error: false,
                retransmit: false,
            },
            1123158610,
            3102381851,
            vec![
                Avp::new(
                    296,
                    AvpFlags {
                        vendor: false,
                        mandatory: true,
                        private: false,
                    },
                    None,
                    AvpType::Integer32(Integer32Avp::new(123456)),
                ),
                Avp::new(
                    264,
                    AvpFlags {
                        vendor: false,
                        mandatory: true,
                        private: false,
                    },
                    Some(10248),
                    AvpType::UTF8String(UTF8StringAvp::new("ses;12345888")),
                ),
            ],
        );

        println!("diameter message: {}", message);
    }
}
