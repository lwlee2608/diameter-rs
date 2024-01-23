use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_xml_rs::{from_str, to_string};
use std::collections::BTreeMap;

use crate::avp::AvpType;

pub struct Definition {
    avps: BTreeMap<u32, AvpDefinition>,
}

pub struct AvpDefinition {
    code: u32,
    name: String,
    avp_type: AvpType,
}

lazy_static! {
    pub static ref DEFAULT_DICT: Definition = {
        let mut definition = Definition::new();
        definition.add_avp(AvpDefinition {
            code: 263,
            name: String::from("Session-Id"),
            avp_type: AvpType::UTF8String,
        });
        definition.add_avp(AvpDefinition {
            code: 264,
            name: String::from("Origin-Host"),
            avp_type: AvpType::Identity,
        });
        definition.add_avp(AvpDefinition {
            code: 296,
            name: String::from("Origin-Realm"),
            avp_type: AvpType::Identity,
        });
        definition.add_avp(AvpDefinition {
            code: 268,
            name: String::from("Result-Code"),
            avp_type: AvpType::Unsigned32,
        });
        definition.add_avp(AvpDefinition {
            code: 415,
            name: String::from("CC-Request-Number"),
            avp_type: AvpType::Unsigned32,
        });
        definition.add_avp(AvpDefinition {
            code: 416,
            name: String::from("CC-Request-Type"),
            avp_type: AvpType::Enumerated,
        });
        definition.add_avp(AvpDefinition {
            code: 30,
            name: String::from("Calling-Station-Id"),
            avp_type: AvpType::UTF8String,
        });
        definition.add_avp(AvpDefinition {
            code: 44,
            name: String::from("Accounting-Session-Id"),
            avp_type: AvpType::OctetString,
        });
        definition.add_avp(AvpDefinition {
            code: 571,
            name: String::from("Timezone-Offset"),
            avp_type: AvpType::Integer32,
        });
        definition.add_avp(AvpDefinition {
            code: 873,
            name: String::from("Service-Information"),
            avp_type: AvpType::Grouped,
        });
        definition.add_avp(AvpDefinition {
            code: 874,
            name: String::from("PS-Information"),
            avp_type: AvpType::Grouped,
        });
        return definition;
    };
}

impl Definition {
    pub fn new() -> Definition {
        Definition {
            avps: BTreeMap::new(),
        }
    }

    pub fn add_avp(&mut self, avp: AvpDefinition) {
        self.avps.insert(avp.code, avp);
    }

    pub fn get_avp(&self, code: u32) -> Option<&AvpDefinition> {
        self.avps.get(&code)
    }

    pub fn get_avp_type(&self, code: u32) -> Option<&AvpType> {
        match self.avps.get(&code) {
            Some(avp) => Some(&avp.avp_type),
            None => None,
        }
    }

    pub fn get_avp_name(&self, code: u32) -> Option<&str> {
        match self.avps.get(&code) {
            Some(avp) => Some(&avp.name),
            None => None,
        }
    }
}

// XML Parsing

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Diameter {
    application: Application,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Application {
    id: String,
    name: String,
    command: Command,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Command {
    code: String,
    short: String,
    name: String,
    request: CommandDetail,
    answer: CommandDetail,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CommandDetail {
    #[serde(rename = "rule", default)]
    rules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Rule {
    avp: String,
    required: String,
    max: Option<String>,
    min: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xml() {
        let xml = r#"
    <diameter>
        <application id="0" name="Base">
            <command code="257" short="CE" name="Capabilities-Exchange">
                <request>
                    <rule avp="Origin-Host" required="true" max="1"/>
                    <rule avp="Origin-Realm" required="true" max="1"/>
                    <rule avp="Host-IP-Address" required="true" min="1"/>
                    <rule avp="Vendor-Id" required="true" max="1"/>
                    <rule avp="Product-Name" required="true" max="1"/>
                    <rule avp="Origin-State-Id" required="false" max="1"/>
                    <rule avp="Supported-Vendor-Id" required="False"/>
                    <rule avp="Auth-Application-Id" required="False"/>
                    <rule avp="Inband-Security-Id" required="False"/>
                    <rule avp="Acct-Application-Id" required="False"/>
                    <rule avp="Vendor-Specific-Application-Id" required="False"/>
                    <rule avp="Firmware-Revision" required="False" max="1"/>
                </request>
                <answer>
                    <rule avp="Result-Code" required="true" max="1"/>
                    <rule avp="Origin-Host" required="true" max="1"/>
                    <rule avp="Origin-Realm" required="true" max="1"/>
                    <rule avp="Host-IP-Address" required="true" min="1"/>
                    <rule avp="Vendor-Id" required="true" max="1"/>
                    <rule avp="Product-Name" required="true" max="1"/>
                    <rule avp="Origin-State-Id" required="false" max="1"/>
                    <rule avp="Error-Message" required="false" max="1"/>
                    <rule avp="Failed-AVP" required="false" max="1"/>
                    <rule avp="Supported-Vendor-Id" required="False"/>
                    <rule avp="Auth-Application-Id" required="False"/>
                    <rule avp="Inband-Security-Id" required="False"/>
                    <rule avp="Acct-Application-Id" required="False"/>
                    <rule avp="Vendor-Specific-Application-Id" required="False"/>
                    <rule avp="Firmware-Revision" required="False" max="1"/>
                </answer>
            </command>
        </application>
    </diameter>
    "#;

        let dict: Diameter = from_str(xml).unwrap();
        println!("dict: {:?}", dict);
    }
}
