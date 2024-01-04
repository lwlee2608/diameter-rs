use crate::avp::Avp;
use crate::diameter::{ApplicationId, CommandCode, DiameterHeader, DiameterMessage};
use std::fmt;

impl fmt::Display for DiameterMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}\n", self.header)?;
        write!(
            f,
            "  {:<40} {:>8} {:>5}  {} {} {}  {:<18}  {}\n",
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
            "{}({}) {}({}) {}{}{}{} {}, {}",
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
        let avp_name = get_avp_name(self.header.code);
        let avp_type = get_avp_type(self.header.code);
        let value = self.value.to_string();
        // let value = get_avp_value(&self.data);

        write!(
            f,
            "  {:<40} {:>8} {:>5}  {} {} {}  {:<18}  {}",
            avp_name,
            self.header.code,
            self.header.code.clone() as u32,
            get_bool_unicode(self.header.flags.vendor),
            get_bool_unicode(self.header.flags.mandatory),
            get_bool_unicode(self.header.flags.private),
            avp_type,
            value
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
        264 => "Origin-Host".to_string(),
        296 => "Origin-Realm".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn get_avp_type(_code: u32) -> String {
    "DiameterIdentity".to_string()
}

// fn get_avp_value(data: &[u8]) -> String {
//     String::from_utf8_lossy(data).to_string()
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avp::integer32::Integer32Avp;
    use crate::avp::{AvpFlags, AvpHeader, AvpType};
    use crate::diameter::CommandFlags;

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
                    // data: vec![0x31, 0x32, 0x33, 0x34, 0x35, 0x36],
                    // type_: AvpType::DiameterIdentity,
                    value: Box::new(Integer32Avp::new(123456)),
                    v: AvpType::Integer32(Integer32Avp::new(123)),
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
                    // data: vec![0x37, 0x38, 0x39, 0x30],
                    // type_: AvpType::DiameterIdentity,
                    value: Box::new(Integer32Avp::new(123456)),
                    v: AvpType::Integer32(Integer32Avp::new(123)),
                },
            ],
        };

        // println!("diameter message: {}", message);
    }
}
