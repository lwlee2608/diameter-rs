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
