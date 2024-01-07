use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct Definition {
    avps: HashMap<u32, AvpDefinition>,
}

pub struct AvpDefinition {
    code: u32,
    name: String,
    // type_: String,
    // flags: String,
    // vendor_id: u32,
    // must: String,
    // may: String,
    // description: String,
}

lazy_static! {
    pub static ref DEFAULT_DICT: Definition = {
        let mut avps = HashMap::new();
        avps.insert(
            264,
            AvpDefinition {
                code: 264,
                name: String::from("Session-Id"),
            },
        );
        avps.insert(
            296,
            AvpDefinition {
                code: 296,
                name: String::from("Origin-Realm"),
            },
        );
        return Definition { avps };
    };
}

impl Definition {
    pub fn new() -> Definition {
        Definition {
            avps: HashMap::new(),
        }
    }

    pub fn add_avp(&mut self, avp: AvpDefinition) {
        self.avps.insert(avp.code, avp);
    }

    pub fn get_avp(&self, code: u32) -> Option<&AvpDefinition> {
        self.avps.get(&code)
    }

    pub fn get_avp_name(&self, code: u32) -> Option<String> {
        match self.avps.get(&code) {
            Some(avp) => Some(avp.name.clone()),
            None => None,
        }
    }
}
