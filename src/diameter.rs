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

#[derive(Debug)]
pub struct DiameterMessage {
    pub header: DiameterHeader,
    pub avps: Vec<Avp>,
}

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

#[derive(Debug, Clone)]
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

#[derive(Debug)]
pub struct CommandFlags {
    pub request: bool,
    pub proxyable: bool,
    pub error: bool,
    pub retransmit: bool,
}

#[derive(Debug, Clone)]
pub enum ApplicationId {
    Common = 0,
    Accounting = 3,
    CreditControl = 4,
    Gx = 16777238,
    Rx = 16777236,
    Sy = 16777302,
}

/*
 * The AVP header.
 *
 * AVP header format:
 *   0                   1                   2                   3
 *   0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                         Command-Code                          |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |  Flags       |                 AVP Length                     |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *  |                         Vendor ID (optional)                  |
 *  +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
 *
 * AVP Flags:
 *   0 1 2 3 4 5 6 7
 *  +-+-+-+-+-+-+-+-+  V(endor), M(andatory), P(rivate)
 *  |V M P r r r r r|  r(eserved)
 *  +-+-+-+-+-+-+-+-+
 *
 */
#[derive(Debug)]
pub struct Avp {
    pub header: AvpHeader,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct AvpHeader {
    pub code: u32,
    pub flags: AvpFlags,
    pub length: u32,
    pub vendor_id: Option<u32>,
}

#[derive(Debug)]
pub struct AvpFlags {
    pub vendor: bool,
    pub mandatory: bool,
    pub private: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diameter_struct() {
        let message = DiameterMessage {
            header: DiameterHeader {
                version: 1,
                length: 64,
                flags: CommandFlags {
                    request: true,
                    proxyable: false,
                    error: false,
                    retransmit: false,
                },
                code: CommandCode::CreditControl,
                application_id: ApplicationId::CreditControl,
                hop_by_hop_id: 1123158610,
                end_to_end_id: 3102381851,
            },
            avps: vec![
                Avp {
                    header: AvpHeader {
                        code: 264,
                        flags: AvpFlags {
                            vendor: false,
                            mandatory: true,
                            private: false,
                        },
                        length: 6,
                        vendor_id: None,
                    },
                    data: vec![0x31, 0x32, 0x33, 0x34, 0x35, 0x36],
                },
                Avp {
                    header: AvpHeader {
                        code: 296,
                        flags: AvpFlags {
                            vendor: false,
                            mandatory: true,
                            private: false,
                        },
                        length: 4,
                        vendor_id: None,
                    },
                    data: vec![0x37, 0x38, 0x39, 0x30],
                },
            ],
        };

        println!("diameter message: {}", message);
    }
}
