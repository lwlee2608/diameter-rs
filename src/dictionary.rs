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
    pub code: u32,
    pub name: String,
    pub avp_type: AvpType,
    pub m_flag: bool,
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

    pub fn get_avp_by_name(&self, name: &str) -> Option<&AvpDefinition> {
        // Might consider indexing avp.name
        self.avps.values().find(|avp| avp.name == name)
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

        let m_flag = match avp.must {
            Some(ref s) if s == "M" => true,
            _ => false,
        };

        let avp_definition = AvpDefinition {
            code: avp.code.parse::<u32>().unwrap(),
            name: avp.name.clone(),
            avp_type,
            m_flag,
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
		<avp name="Acct-Interim-Interval" code="85" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Accounting-Realtime-Required" code="483" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Enumerated">
				<item code="1" name="DELIVER_AND_GRANT"/>
				<item code="2" name="GRANT_AND_STORE"/>
				<item code="3" name="GRANT_AND_LOSE"/>
			</data>
		</avp>

		<avp name="Acct-Multi-Session-Id" code="50" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="UTF8String"/>
		</avp>

		<avp name="Accounting-Record-Number" code="485" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Accounting-Record-Type" code="480" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Enumerated">
				<item code="1" name="EVENT_RECORD"/>
				<item code="2" name="START_RECORD"/>
				<item code="3" name="INTERIM_RECORD"/>
				<item code="4" name="STOP_RECORD"/>
			</data>
		</avp>

		<avp name="Accounting-Session-Id" code="44" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="OctetString"/>
		</avp>

		<avp name="Accounting-Sub-Session-Id" code="287" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Unsigned64"/>
		</avp>

		<avp name="Acct-Application-Id" code="259" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Auth-Application-Id" code="258" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Auth-Request-Type" code="274" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Enumerated">
				<item code="1" name="AUTHENTICATE_ONLY"/>
				<item code="2" name="AUTHORIZE_ONLY"/>
				<item code="3" name="AUTHORIZE_AUTHENTICATE"/>
			</data>
		</avp>

		<avp name="Authorization-Lifetime" code="291" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Auth-Grace-Period" code="276" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Auth-Session-State" code="277" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Enumerated">
				<item code="0" name="STATE_MAINTAINED"/>
				<item code="1" name="NO_STATE_MAINTAINED"/>
			</data>
		</avp>

		<avp name="Re-Auth-Request-Type" code="285" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Enumerated">
				<item code="0" name="AUTHORIZE_ONLY"/>
				<item code="1" name="AUTHORIZE_AUTHENTICATE"/>
			</data>
		</avp>

		<avp name="Class" code="25" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="OctetString"/>
		</avp>

		<avp name="Destination-Host" code="293" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="DiameterIdentity"/>
		</avp>

		<avp name="Destination-Realm" code="283" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="DiameterIdentity"/>
		</avp>

		<avp name="Disconnect-Cause" code="273" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Enumerated">
				<item code="0" name="REBOOTING"/>
				<item code="1" name="BUSY"/>
				<item code="2" name="DO_NOT_WANT_TO_TALK_TO_YOU"/>
			</data>
		</avp>

		<avp name="Error-Message" code="281" must="-" may="P" must-not="V,M" may-encrypt="-">
			<data type="UTF8String"/>
		</avp>

		<avp name="Error-Reporting-Host" code="294" must="-" may="P" must-not="V,M" may-encrypt="-">
			<data type="DiameterIdentity"/>
		</avp>

		<avp name="Event-Timestamp" code="55" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Time"/>
		</avp>

		<avp name="Experimental-Result" code="297" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Grouped">
				<rule avp="Vendor-Id" required="true" max="1"/>
				<rule avp="Experimental-Result-Code" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Experimental-Result-Code" code="298" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Failed-AVP" code="279" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Grouped"/>
		</avp>

		<avp name="Firmware-Revision" code="267" must="-" may="-" must-not="P,V,M" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Host-IP-Address" code="257" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Address"/>
		</avp>

		<avp name="Inband-Security-Id" code="299" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Multi-Round-Time-Out" code="272" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Origin-Host" code="264" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="DiameterIdentity"/>
		</avp>

		<avp name="Origin-Realm" code="296" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="DiameterIdentity"/>
		</avp>

		<avp name="Origin-State-Id" code="278" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Product-Name" code="269" must="-" may="-" must-not="P,V,M" may-encrypt="-">
			<data type="UTF8String"/>
		</avp>

		<avp name="Proxy-Host" code="280" must="M" may="-" must-not="P,V" may-encrypt="-">
			<data type="DiameterIdentity"/>
		</avp>

		<avp name="Proxy-Info" code="284" must="M" may="-" must-not="P,V" may-encrypt="-">
			<data type="Grouped">
				<rule avp="Proxy-Host" required="true" max="1"/>
				<rule avp="Proxy-State" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Proxy-State" code="33" must="M" may="-" must-not="P,V" may-encrypt="-">
			<data type="OctetString"/>
		</avp>

		<avp name="Redirect-Host" code="292" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="DiameterURI"/>
		</avp>

		<avp name="Redirect-Host-Usage" code="261" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Enumerated">
				<item code="0" name="DONT_CACHE"/>
				<item code="1" name="ALL_SESSION"/>
				<item code="2" name="ALL_REALM"/>
				<item code="3" name="REALM_AND_APPLICATION"/>
				<item code="4" name="ALL_APPLICATION"/>
				<item code="5" name="ALL_HOST"/>
				<item code="6" name="ALL_USER"/>
			</data>
		</avp>

		<avp name="Redirect-Max-Cache-Time" code="262" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Result-Code" code="268" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Route-Record" code="282" must="M" may="-" must-not="P,V" may-encrypt="-">
			<data type="DiameterIdentity"/>
		</avp>

		<avp name="Session-Id" code="263" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="UTF8String"/>
		</avp>

		<avp name="Session-Timeout" code="27" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Session-Binding" code="270" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Session-Server-Failover" code="271" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Enumerated">
				<item code="0" name="REFUSE_SERVICE"/>
				<item code="1" name="TRY_AGAIN"/>
				<item code="2" name="ALLOW_SERVICE"/>
				<item code="3" name="TRY_AGAIN_ALLOW_SERVICE"/>
			</data>
		</avp>

		<avp name="Supported-Vendor-Id" code="265" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Termination-Cause" code="295" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Enumerated">
				<item code="1" name="DIAMETER_LOGOUT"/>
				<item code="2" name="DIAMETER_SERVICE_NOT_PROVIDED"/>
				<item code="3" name="DIAMETER_BAD_ANSWER"/>
				<item code="4" name="DIAMETER_ADMINISTRATIVE"/>
				<item code="5" name="DIAMETER_LINK_BROKEN"/>
				<item code="6" name="DIAMETER_AUTH_EXPIRED"/>
				<item code="7" name="DIAMETER_USER_MOVED"/>
				<item code="8" name="DIAMETER_SESSION_TIMEOUT"/>
			</data>
		</avp>

		<avp name="User-Name" code="1" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="UTF8String"/>
		</avp>

		<avp name="Vendor-Id" code="266" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Unsigned32"/>
		</avp>

		<avp name="Vendor-Specific-Application-Id" code="260" must="M" may="P" must-not="V" may-encrypt="-">
			<data type="Grouped">
				<rule avp="Vendor-Id" required="false" max="1"/>
				<rule avp="Auth-Application-Id" required="true" max="1"/>
				<rule avp="Acct-Application-Id" required="true" max="1"/>
			</data>
		</avp>

		<!-- IETF RFC 7683 - https://tools.ietf.org/html/rfc7683 -->
		<avp name="OC-Supported-Features" code="621" must-not="V">
			<data type="Grouped">
				<rule avp="OC-Feature-Vector" required="false"/>
				<rule avp="AVP" required="false"/>
			</data>
		</avp>

		<avp name="OC-Feature-Vector" code="622" must-not="V">
			<data type="Unsigned64"/>
		</avp>

		<avp name="OC-OLR" code="623" must-not="V">
			<data type="Grouped">
				<rule avp="OC-Sequence-Number" required="true" max="1"/>
				<rule avp="OC-Report-Type" required="true" max="1"/>
				<rule avp="OC-Reduction-Percentage" required="false" max="1"/>
				<rule avp="OC-Validity-Duration" required="false" max="1"/>
				<rule avp="AVP" required="false"/>
			</data>
		</avp>

		<avp name="OC-Sequence-Number" code="624" must-not="V">
			<data type="Unsigned64"/>
		</avp>

		<avp name="OC-Validity-Duration" code="625" must-not="V">
			<data type="Unsigned32"/>
		</avp>

		<avp name="OC-Report-Type" code="626" must-not="V">
			<data type="Enumerated">
				<item code="0" name="HOST_REPORT"/>
				<item code="1" name="REALM_REPORT"/>
			</data>
		</avp>

		<avp name="OC-Reduction-Percentage" code="627" must-not="V">
			<data type="Unsigned32"/>
		</avp>

		<!-- IETF RFC 7944 - https://tools.ietf.org/html/rfc7944 -->
		<avp name="DRMP" code="301" must-not="V">
			<data type="Enumerated">
				<item code="0" name="PRIORITY_0"/>
				<item code="1" name="PRIORITY_1"/>
				<item code="2" name="PRIORITY_2"/>
				<item code="3" name="PRIORITY_3"/>
				<item code="4" name="PRIORITY_4"/>
				<item code="5" name="PRIORITY_5"/>
				<item code="6" name="PRIORITY_6"/>
				<item code="7" name="PRIORITY_7"/>
				<item code="8" name="PRIORITY_8"/>
				<item code="9" name="PRIORITY_9"/>
				<item code="10" name="PRIORITY_10"/>
				<item code="11" name="PRIORITY_11"/>
				<item code="12" name="PRIORITY_12"/>
				<item code="13" name="PRIORITY_13"/>
				<item code="14" name="PRIORITY_14"/>
				<item code="15" name="PRIORITY_15"/>
			</data>
		</avp>
		
		<avp name="CC-Request-Number" code="415" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.2 -->
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
	
		<avp name="CC-Correlation-Id" code="411" must="-" may="P,M" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.1 -->
			<data type="OctetString"/>
		</avp>

		<avp name="CC-Input-Octets" code="412" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.24 -->
			<data type="Unsigned64"/>
		</avp>

		<avp name="CC-Money" code="413" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.22 -->
			<data type="Grouped">
				<rule avp="Unit-Value" required="true" max="1"/>
				<rule avp="Currency-Code" required="true" max="1"/>
			</data>
		</avp>

		<avp name="CC-Output-Octets" code="414" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.25 -->
			<data type="Unsigned64"/>
		</avp>

		<avp name="CC-Request-Number" code="415" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.2 -->
			<data type="Unsigned32"/>
		</avp>

		<avp name="CC-Request-Type" code="416" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.3 -->
			<data type="Enumerated">
				<item code="1" name="INITIAL_REQUEST"/>
				<item code="2" name="UPDATE_REQUEST"/>
				<item code="3" name="TERMINATION_REQUEST"/>
			</data>
		</avp>

		<avp name="CC-Service-Specific-Units" code="417" must="M" may="P" must-not="V" may-encrypt="Y">
			<data type="Unsigned64"/>
		</avp>

		<avp name="CC-Session-Failover" code="418" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.4 -->
			<data type="Enumerated">
				<item code="0" name="FAILOVER_NOT_SUPPORTED"/>
				<item code="1" name="FAILOVER_SUPPORTED"/>
			</data>
		</avp>

		<avp name="CC-Sub-Session-Id" code="419" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.5 -->
			<data type="Unsigned64"/>
		</avp>

		<avp name="CC-Time" code="420" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.21 -->
			<data type="Unsigned32"/>
		</avp>

		<avp name="CC-Total-Octets" code="421" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.23 -->
			<data type="Unsigned64"/>
		</avp>

		<avp name="CC-Unit-Type" code="454" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.32 -->
			<data type="Enumerated">
				<item code="0" name="TIME"/>
				<item code="1" name="MONEY"/>
				<item code="2" name="TOTAL-OCTETS"/>
				<item code="3" name="INPUT-OCTETS"/>
				<item code="4" name="OUTPUT-OCTETS"/>
				<item code="5" name="SERVICE-SPECIFIC-UNITS"/>
			</data>
		</avp>

		<avp name="Check-Balance-Result" code="422" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.6 -->
			<data type="Enumerated">
				<item code="0" name="ENOUGH_CREDIT"/>
				<item code="1" name="NO_CREDIT"/>
			</data>
		</avp>

		<avp name="Cost-Information" code="423" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.7 -->
			<data type="Grouped">
				<rule avp="Unit-Value" required="true" max="1"/>
				<rule avp="Currency-Code" required="true" max="1"/>
				<rule avp="Cost-Unit" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Cost-Unit" code="424" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.12 -->
			<data type="UTF8String"/>
		</avp>

		<avp name="Credit-Control" code="426" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.13 -->
			<data type="Enumerated">
				<item code="0" name="CREDIT_AUTHORIZATION"/>
				<item code="1" name="RE_AUTHORIZATION"/>
			</data>
		</avp>

		<avp name="Credit-Control-Failure-Handling" code="427" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.14 -->
			<data type="Enumerated">
				<item code="0" name="TERMINATE"/>
				<item code="1" name="CONTINUE"/>
				<item code="2" name="RETRY_AND_TERMINATE"/>
			</data>
		</avp>

		<avp name="Currency-Code" code="425" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.11 -->
			<data type="Unsigned32"/>
		</avp>

		<avp name="Direct-Debiting-Failure-Handling" code="428" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.15 -->
			<data type="Enumerated">
				<item code="0" name="TERMINATE_OR_BUFFER"/>
				<item code="1" name="CONTINUE"/>
			</data>
		</avp>

		<avp name="Exponent" code="429" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.9 -->
			<data type="Integer32"/>
		</avp>

		<avp name="Final-Unit-Action" code="449" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.35 -->
			<data type="Enumerated">
				<item code="0" name="TERMINATE"/>
				<item code="1" name="REDIRECT"/>
				<item code="2" name="RESTRICT_ACCESS"/>
			</data>
		</avp>

		<avp name="Final-Unit-Indication" code="430" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.34 -->
			<data type="Grouped">
				<rule avp="Final-Unit-Action" required="true" max="1"/>
				<rule avp="Restriction-Filter-Rule" required="false" max="1"/>
				<rule avp="Filter-Id" required="false" max="1"/>
				<rule avp="Redirect-Server" required="false" max="1"/>
			</data>
		</avp>

		<avp name="Granted-Service-Unit" code="431" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.17 -->
			<data type="Grouped">
				<rule avp="Tariff-Time-Change" required="false" max="1"/>
				<rule avp="CC-Time" required="false" max="1"/>
				<rule avp="CC-Money" required="false" max="1"/>
				<rule avp="CC-Total-Octets" required="false" max="1"/>
				<rule avp="CC-Input-Octets" required="false" max="1"/>
				<rule avp="CC-Output-Octets" required="false" max="1"/>
				<rule avp="CC-Service-Specific-Units" required="false" max="1"/>
				<!-- *[ AVP ]-->
			</data>
		</avp>

		<avp name="G-S-U-Pool-Identifier" code="453" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.31 -->
			<data type="Unsigned32"/>
		</avp>

		<avp name="G-S-U-Pool-Reference" code="457" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.30 -->
			<data type="Grouped">
				<rule avp="G-S-U-Pool-Identifier" required="true" max="1"/>
				<rule avp="CC-Unit-Type" required="true" max="1"/>
				<rule avp="Unit-Value" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Multiple-Services-Credit-Control" code="456" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.16 -->
			<data type="Grouped">
				<rule avp="Granted-Service-Unit" required="false" max="1"/>
				<rule avp="Requested-Service-Unit" required="false" max="1"/>
				<rule avp="Used-Service-Unit" required="false" max="1"/>
				<rule avp="Tariff-Change-Usage" required="false" max="1"/>
				<rule avp="Service-Identifier" required="false" max="1"/>
				<rule avp="Rating-Group" required="false" max="1"/>
				<rule avp="G-S-U-Pool-Reference" required="false" max="1"/>
				<rule avp="Validity-Time" required="false" max="1"/>
				<rule avp="Result-Code" required="false" max="1"/>
				<rule avp="Final-Unit-Indication" required="false" max="1"/>
				<!-- *[ AVP ]-->
			</data>
		</avp>

		<avp name="Multiple-Services-Indicator" code="455" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.40 -->
			<data type="Enumerated">
				<item code="0" name="MULTIPLE_SERVICES_NOT_SUPPORTED"/>
				<item code="1" name="MULTIPLE_SERVICES_SUPPORTED"/>
			</data>
		</avp>

		<avp name="Rating-Group" code="432" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.29 -->
			<data type="Unsigned32"/>
		</avp>

		<avp name="Redirect-Address-Type" code="433" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.38 -->
			<data type="Enumerated">
				<item code="0" name="IPv4 Address"/>
				<item code="1" name="IPv6 Address"/>
				<item code="2" name="URL"/>
				<item code="3" name="SIP URI"/>
			</data>
		</avp>

		<avp name="Redirect-Server" code="434" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.37 -->
			<data type="Grouped">
				<rule avp="Redirect-Address-Type" required="true" max="1"/>
				<rule avp="Redirect-Server-Address" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Redirect-Server-Address" code="435" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.39 -->
			<data type="UTF8String"/>
		</avp>

		<avp name="Requested-Action" code="436" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.41 -->
			<data type="Enumerated">
				<item code="0" name="DIRECT_DEBITING"/>
				<item code="1" name="REFUND_ACCOUNT"/>
				<item code="2" name="CHECK_BALANCE"/>
				<item code="3" name="PRICE_ENQUIRY"/>
			</data>
		</avp>

		<avp name="Requested-Service-Unit" code="437" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.18-->
			<data type="Grouped">
				<rule avp="CC-Time" required="false" max="1"/>
				<rule avp="CC-Money" required="false" max="1"/>
				<rule avp="CC-Total-Octets" required="false" max="1"/>
				<rule avp="CC-Input-Octets" required="false" max="1"/>
				<rule avp="CC-Output-Octets" required="false" max="1"/>
				<rule avp="CC-Service-Specific-Units" required="false" max="1"/>
				<!-- *[ AVP ]-->
			</data>
		</avp>

		<avp name="Restriction-Filter-Rule" code="438" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.36-->
			<data type="IPFilterRule"/>
		</avp>

		<avp name="Service-Context-Id" code="461" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.42-->
			<data type="UTF8String"/>
		</avp>

		<avp name="Service-Identifier" code="439" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.28-->
			<data type="Unsigned32"/>
		</avp>

		<avp name="Service-Parameter-Info" code="440" must="-" may="P,M" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.43-->
			<data type="Grouped">
				<rule avp="Service-Parameter-Type" required="true" max="1"/>
				<rule avp="Service-Parameter-Value" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Service-Parameter-Type" code="441" must="-" may="P,M" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.44-->
			<data type="Unsigned32"/>
		</avp>

		<avp name="Service-Parameter-Value" code="442" must="-" may="P,M" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.45-->
			<data type="OctetString"/>
		</avp>

		<avp name="Subscription-Id" code="443" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.46-->
			<data type="Grouped">
				<rule avp="Subscription-Id-Type" required="true" max="1"/>
				<rule avp="Subscription-Id-Data" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Subscription-Id-Data" code="444" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.48-->
			<data type="UTF8String"/>
		</avp>

		<avp name="Subscription-Id-Type" code="450" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.47-->
			<data type="Enumerated">
				<item code="0" name="END_USER_E164"/>
				<item code="1" name="END_USER_IMSI"/>
				<item code="2" name="END_USER_SIP_URI"/>
				<item code="3" name="END_USER_NAI"/>
			</data>
		</avp>

		<avp name="Tariff-Change-Usage" code="452" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.27-->
			<data type="Enumerated">
				<item code="0" name="UNIT_BEFORE_TARIFF_CHANGE"/>
				<item code="1" name="UNIT_AFTER_TARIFF_CHANGE"/>
				<item code="2" name="UNIT_INDETERMINATE"/>
			</data>
		</avp>

		<avp name="Tariff-Time-Change" code="451" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.20-->
			<data type="Time"/>
		</avp>

		<avp name="Unit-Value" code="445" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.8-->
			<data type="Grouped">
				<rule avp="Value-Digits" required="true" max="1"/>
				<rule avp="Exponent" required="true" max="1"/>
			</data>
		</avp>

		<avp name="Used-Service-Unit" code="446" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.19-->
			<data type="Grouped">
				<rule avp="Tariff-Change-Usage" required="false" max="1"/>
				<rule avp="CC-Time" required="false" max="1"/>
				<rule avp="CC-Money" required="false" max="1"/>
				<rule avp="CC-Total-Octets" required="false" max="1"/>
				<rule avp="CC-Input-Octets" required="false" max="1"/>
				<rule avp="CC-Output-Octets" required="false" max="1"/>
				<rule avp="CC-Service-Specific-Units" required="false" max="1"/>
				<!-- *[ AVP ]-->
			</data>
		</avp>

		<avp name="User-Equipment-Info" code="458" must="-" may="P,M" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.49-->
			<data type="Grouped">
				<rule avp="User-Equipment-Info-Type" required="true" max="1"/>
				<rule avp="User-Equipment-Info-Value" required="true" max="1"/>
			</data>
		</avp>

		<avp name="User-Equipment-Info-Type" code="459" must="-" may="P,M" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.50-->
			<data type="Enumerated">
				<item code="0" name="IMEISV"/>
				<item code="1" name="MAC"/>
				<item code="2" name="EUI64"/>
				<item code="3" name="MODIFIED_EUI64"/>
			</data>
		</avp>

		<avp name="User-Equipment-Info-Value" code="460" must="-" may="P,M" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.51-->
			<data type="OctetString"/>
		</avp>

		<avp name="Value-Digits" code="447" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.10-->
			<data type="Integer64"/>
		</avp>

		<avp name="Validity-Time" code="448" must="M" may="P" must-not="V" may-encrypt="Y">
			<!-- http://tools.ietf.org/html/rfc4006#section-8.33-->
			<data type="Unsigned32"/>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dict() {
        let dict = &DEFAULT_DICT;
        assert_eq!(dict.get_avp(416).unwrap().name, "CC-Request-Type");
        assert_eq!(dict.get_avp(264).unwrap().name, "Origin-Host");
        assert_eq!(dict.get_avp(263).unwrap().name, "Session-Id");
        assert_eq!(dict.get_avp(1).unwrap().name, "User-Name");
        assert_eq!(dict.get_avp(258).unwrap().name, "Auth-Application-Id");

        println!("Total AVP definitions {}", dict.avps.len());
    }
}
