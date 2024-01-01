use crate::diameter::{ApplicationId, Avp, CommandCode, DiameterHeader, DiameterMessage};
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
        write!(
            f,
            "{}({}) {}({}) {}{}{}{} {}, {}",
            self.code,
            self.code.clone() as u32,
            self.application_id,
            self.application_id.clone() as u32,
            if self.flags.request {
                "request "
            } else {
                "answer "
            },
            if self.flags.error { "error " } else { "" },
            if self.flags.proxyable {
                "proxiable "
            } else {
                ""
            },
            if self.flags.retransmit {
                "retransmit "
            } else {
                ""
            },
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
        let value = get_avp_value(&self.data);

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

fn get_avp_value(data: &[u8]) -> String {
    String::from_utf8_lossy(data).to_string()
}
