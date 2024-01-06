use crate::avp::Avp;
use crate::diameter::{ApplicationId, CommandCode, DiameterHeader, DiameterMessage};
use std::fmt;

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
    use super::*;
    use crate::avp::integer32::Integer32Avp;
    use crate::avp::utf8string::UTF8StringAvp;
    use crate::avp::{AvpFlags, AvpType};
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
        };

        println!("diameter message: {}", message);
    }
}
