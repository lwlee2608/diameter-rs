use lazy_static::lazy_static;
use serde::Deserialize;
use serde_xml_rs::from_str;
use std::collections::BTreeMap;

use crate::avp::AvpType;

#[derive(Debug)]
pub struct Definition {
    avps: BTreeMap<u32, AvpDefinition>,
}

#[derive(Debug)]
pub struct AvpDefinition {
    code: u32,
    name: String,
    avp_type: AvpType,
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

#[derive(Debug, Deserialize, PartialEq)]
struct Diameter {
    application: Application,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Application {
    id: String,
    name: String,
    command: Option<Command>,
    #[serde(rename = "avp", default)]
    avps: Vec<Avp>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Command {
    code: String,
    short: String,
    name: String,
    request: CommandDetail,
    answer: CommandDetail,
}

#[derive(Debug, Deserialize, PartialEq)]
struct CommandDetail {
    #[serde(rename = "rule", default)]
    rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Rule {
    avp: String,
    required: String,
    max: Option<String>,
    min: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Avp {
    name: String,
    code: String,
    must: Option<String>,
    may: Option<String>,
    #[serde(rename = "must-not")]
    must_not: Option<String>,
    #[serde(rename = "may-encrypt")]
    may_encrypt: Option<String>,
    data: Data,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Data {
    #[serde(rename = "type")]
    data_type: String,
    #[serde(default)]
    item: Vec<Item>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Item {
    code: String,
    name: String,
}

pub fn parse(xml: &str) -> Definition {
    let dict: Diameter = from_str(xml).unwrap();

    let mut definition = Definition::new();

    dict.application.avps.iter().for_each(|avp| {
        let avp_type = match avp.data.data_type.as_str() {
            "UTF8String" => AvpType::UTF8String,
            "OctetString" => AvpType::OctetString,
            "Integer32" => AvpType::Integer32,
            "Integer64" => AvpType::Integer64,
            "Unsigned32" => AvpType::Unsigned32,
            "Unsigned64" => AvpType::Unsigned64,
            "Enumerated" => AvpType::Enumerated,
            "Grouped" => AvpType::Grouped,
            "DiameterIdentity" => AvpType::Identity,
            "DiameterURI" => AvpType::DiameterURI,
            "Time" => AvpType::Time,
            "Address" => AvpType::Address,
            "IPv4" => AvpType::AddressIPv4,
            "IPv6" => AvpType::AddressIPv6,
            "Float32" => AvpType::Float32,
            "Float64" => AvpType::Float64,
            _ => AvpType::Unknown,
        };

        let avp_definition = AvpDefinition {
            code: avp.code.parse::<u32>().unwrap(),
            name: avp.name.clone(),
            avp_type,
        };

        definition.add_avp(avp_definition);
    });

    definition
}

lazy_static! {
    pub static ref DEFAULT_DICT: Definition = {
        let xml = &DEFAULT_DICT_XML;
        parse(xml)
    };
    pub static ref DEFAULT_DICT_XML: &'static str = {
        let xml = r#"
<diameter>
    <application id="0" name="Base">
		<avp name="Session-Id" code="263" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="UTF8String"/>
		</avp>

		<avp name="Origin-Host" code="264" must="M" may="P" must-not="V" may-encrypt="-">
            <data type="DiameterIdentity"/>
        </avp>

		<avp name="CC-Request-Number" code="415" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.2 -->
			<data type="Unsigned32"/>
		</avp>

		<avp name="Origin-Realm" code="296" must="M" may="P" must-not="V" may-encrypt="-">
            		<data type="DiameterIdentity"/>
        	</avp>

		<avp name="Destination-Host" code="293" must="M" may="P" must-not="V" may-encrypt="-">
            <data type="DiameterIdentity"/>
        </avp>

        <avp name="Destination-Realm" code="283" must="M" may="P" must-not="V" may-encrypt="-">
            <data type="DiameterIdentity"/>
        </avp>
		
		<avp name="Auth-Application-Id" code="258" must="M" may="P" must-not="V" may-encrypt="-">
            <data type="Unsigned32"/>
        </avp>

		 <avp name="Result-Code" code="268" must="M" may="P" must-not="V" may-encrypt="-">
            <data type="Unsigned32"/>
        </avp>

		<avp name="Service-Information" code="873" must="V,M" may="P" must-not="-" may-encrypt="N" vendor-id="10415">
			<data type="Grouped">
				<rule avp="Subscription-Id" required="false"/>
				<rule avp="AoC-Information" required="false" max="1"/>
				<rule avp="PS-Information" required="false" max="1"/>
				<rule avp="IMS-Information" required="false" max="1"/>
				<rule avp="MMS-Information" required="false" max="1"/>
				<rule avp="LCS-Information" required="false" max="1"/>
				<rule avp="PoC-Information" required="false" max="1"/>
				<rule avp="MBMS-Information" required="false" max="1"/>
				<rule avp="SMS-Information" required="false" max="1"/>
				<rule avp="VCS-Information" required="false" max="1"/>
				<rule avp="MMTel-Information" required="false" max="1"/>
				<rule avp="Service-Generic-Information" required="false" max="1"/>
				<rule avp="IM-Information" required="false" max="1"/>
				<rule avp="DCD-Information" required="false" max="1"/>
			</data>
		</avp>

		<avp name="PS-Information" code="874" must="V,M" may="P" must-not="-" may-encrypt="N" vendor-id="10415">
			<data type="Grouped">
				<rule avp="TGPP-Charging-Id" required="false" max="1"/>
				<rule avp="PDN-Connection-Charging-Id" required="false" max="1"/>
				<rule avp="Node-Id" required="false" max="1"/>
				<rule avp="TGPP-PDP-Type" required="false" max="1"/>
				<rule avp="PDP-Address" required="false"/>
				<rule avp="PDP-Address-Prefix-Length" required="false" max="1"/>
				<rule avp="Dynamic-Address-Flag" required="false" max="1"/>
				<rule avp="Dynamic-Address-Flag-Extension" required="false" max="1"/>
				<rule avp="QoS-Information" required="false" max="1"/>
				<rule avp="SGSN-Address" required="false"/>
				<rule avp="GGSN-Address" required="false"/>
				<rule avp="TDF-IP-Address" required="false"/>
				<rule avp="SGW-Address" required="false"/>
				<rule avp="ePDG-Address" required="false"/>
				<rule avp="CG-Address" required="false" max="1"/>
				<rule avp="Serving-Node-Type" required="false" max="1"/>
				<rule avp="SGW-Change" required="false" max="1"/>
				<rule avp="TGPP-IMSI-MCC-MNC" required="false" max="1"/>
				<rule avp="IMSI-Unauthenticated-Flag" required="false" max="1"/>
				<rule avp="TGPP-GGSN-MCC-MNC" required="false" max="1"/>
				<rule avp="TGPP-NSAPI" required="false" max="1"/>
				<rule avp="Called-Station-Id" required="false" max="1"/>
				<rule avp="TGPP-Session-Stop-Indicator" required="false" max="1"/>
				<rule avp="TGPP-Selection-Mode" required="false" max="1"/>
				<rule avp="TGPP-Charging-Characteristics" required="false" max="1"/>
				<rule avp="Charging-Characteristics-Selection-Mode" required="false" max="1"/>
				<rule avp="TGPP-SGSN-MCC-MNC" required="false" max="1"/>
				<rule avp="TGPP-MS-TimeZone" required="false" max="1"/>
				<rule avp="Charging-Rule-Base-Name" required="false" max="1"/>
				<rule avp="ADC-Rule-Base-Name" required="false" max="1"/>
				<rule avp="TGPP-User-Location-Info" required="false" max="1"/>
				<rule avp="User-Location-Info-Time" required="false" max="1"/>
				<rule avp="User-CSG-Information" required="false" max="1"/>
				<rule avp="Presence-Reporting-Area-Information" required="false" max="1"/>
				<rule avp="TGPP2-BSID" required="false" max="1"/>
				<rule avp="TWAN-User-Location-Info" required="false" max="1"/>
				<rule avp="TGPP-RAT-Type" required="false" max="1"/>
				<rule avp="PS-Furnish-Charging-Information" required="false" max="1"/>
				<rule avp="PDP-Context-Type" required="false" max="1"/>
				<rule avp="Offline-Charging" required="false" max="1"/>
				<rule avp="Traffic-Data-Volumes" required="false"/>
				<rule avp="Service-Data-Container" required="false"/>
				<rule avp="User-Equipment-Info" required="false" max="1"/>
				<rule avp="Terminal-Information" required="false" max="1"/>
				<rule avp="Start-Time" required="false" max="1"/>
				<rule avp="Stop-Time" required="false" max="1"/>
				<rule avp="Change-Condition" required="false" max="1"/>
				<rule avp="Diagnostics" required="false" max="1"/>
				<rule avp="Low-Priority-Indicator" required="false" max="1"/>
				<rule avp="MME-Number-for-MT-SMS" required="false" max="1"/>
				<rule avp="MME-Name" required="false" max="1"/>
				<rule avp="MME-Realm" required="false" max="1"/>
				<rule avp="Logical-Access-Id" required="false" max="1"/>
				<rule avp="Physical-Access-Id" required="false" max="1"/>
				<rule avp="Fixed-User-Location-Info" required="false" max="1"/>
				<rule avp="CN-Operator-Selection-Entity" required="false" max="1"/>
			</data>
		</avp>

		<avp name="Called-Station-Id" code="30" must="M" may="-" must-not="V" may-encrypt="Y">
            <data type="UTF8String"/>
        </avp>
	
		<avp name="CC-Request-Type" code="416" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.3 -->
			<data type="Enumerated">
				<item code="1" name="INITIAL_REQUEST"/>
				<item code="2" name="UPDATE_REQUEST"/>
				<item code="3" name="TERMINATION_REQUEST"/>
			</data>
		</avp>

		<avp name="Timezone-Offset" code="571" vendor-id="10415" must="V" may-encrypt="Y">
			<data type="Integer32"/>
		</avp>
		
    </application>
</diameter>
    "#;
        xml
    };
}
