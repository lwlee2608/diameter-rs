use lazy_static::lazy_static;
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
