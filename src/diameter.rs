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
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::error::Error;
use std::io::Read;

#[derive(Debug)]
pub struct DiameterMessage {
    pub header: DiameterHeader,
    pub avps: Vec<Avp>,
}

const HEADER_LENGTH: u32 = 20;

#[derive(Debug)]
pub struct DiameterHeader {
    pub version: u8,
    pub length: u32,
    pub flags: CommandFlags,
    pub code: CommandCode,
    pub application_id: ApplicationId,
    pub hop_by_hop_id: u32,
    pub end_to_end_id: u32,
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

#[rustfmt::skip]
impl DiameterHeader {
    // pub fn decode_from<'a>(b: &'a [u8]) -> Result<DiameterHeader, Box<dyn Error>> {
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<DiameterHeader, Box<dyn Error>> {
        let mut b = [0; HEADER_LENGTH as usize];
        reader.read_exact(&mut b)?;

        if b.len() < HEADER_LENGTH as usize {
            return Err("Invalid diameter header, too short".into());
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

impl DiameterMessage {
    // pub fn decode_from<'a>(b: &'a [u8]) -> Result<DiameterMessage, Box<dyn Error>> {
    pub fn decode_from<R: Read>(reader: &mut R) -> Result<DiameterMessage, Box<dyn Error>> {
        let header = DiameterHeader::decode_from(reader)?;
        let mut avps = Vec::new();

        let total_length = header.length;
        let mut offset = HEADER_LENGTH;
        while offset < total_length {
            let avp = Avp::decode_from(reader)?;
            offset += avp.header.length;
            avps.push(avp);
        }

        // sanity check, make sure everything is read
        if offset != total_length {
            return Err("Invalid diameter message, length mismatch".into());
        }

        Ok(DiameterMessage { header, avps })
    }
}

#[cfg(test)]
mod tests {
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
}
