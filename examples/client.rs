use diameter::avp;
use diameter::avp::address::Value::IPv4;
use diameter::avp::flags::M;
use diameter::avp::Address;
use diameter::avp::Avp;
use diameter::avp::Enumerated;
use diameter::avp::Identity;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::dictionary;
use diameter::dictionary::Dictionary;
use diameter::flags;
use diameter::transport::DiameterClient;
use diameter::transport::DiameterClientConfig;
use diameter::{ApplicationId, CommandCode, DiameterMessage};
use std::fs;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load dictionary
    let mut dict = Dictionary::new();
    dict.load_xml(&fs::read_to_string("dict/3gpp-ro-rf.xml").unwrap());
    dict.load_xml(&dictionary::DEFAULT_DICT_XML);
    let dict = Arc::new(dict);

    // Initialize a Diameter client and connect it to the server
    let client_config = DiameterClientConfig {
        use_tls: false,
        verify_cert: false,
    };
    let mut client = DiameterClient::new("localhost:3868", client_config);
    let mut handler = client.connect().await.unwrap();
    let dict_ref = Arc::clone(&dict);
    tokio::spawn(async move {
        DiameterClient::handle(&mut handler, dict_ref).await;
    });

    // Send a Capabilities-Exchange-Request (CER) Diameter message
    send_cer(&mut client, Arc::clone(&dict)).await;

    // Send a Credit-Control-Request (CCR) Diameter message
    send_ccr(&mut client, Arc::clone(&dict)).await;
}

async fn send_cer(client: &mut DiameterClient, dict: Arc<Dictionary>) {
    let seq_num = client.get_next_seq_num();
    let mut cer = DiameterMessage::new(
        CommandCode::CapabilitiesExchange,
        ApplicationId::Common,
        flags::REQUEST,
        seq_num,
        seq_num,
        dict,
    );
    cer.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
    cer.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
    cer.add_avp(avp!(
        257,
        None,
        M,
        Address::new(IPv4(Ipv4Addr::new(127, 0, 0, 1)))
    ));
    cer.add_avp(avp!(266, None, M, Unsigned32::new(35838)));
    cer.add_avp(avp!(269, None, M, UTF8String::new("diameter-rs")));

    let resp = client.send_message(cer).await.unwrap();
    let cea = resp.await.unwrap();
    log::info!("Received rseponse: {}", cea);
}

async fn send_ccr(client: &mut DiameterClient, dict: Arc<Dictionary>) {
    let seq_num = client.get_next_seq_num();
    let mut ccr = DiameterMessage::new(
        CommandCode::CreditControl,
        ApplicationId::CreditControl,
        flags::REQUEST,
        seq_num,
        seq_num,
        dict,
    );
    ccr.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
    ccr.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
    ccr.add_avp(avp!(263, None, M, UTF8String::new("ses;12345888")));
    ccr.add_avp(avp!(416, None, M, Enumerated::new(1)));
    ccr.add_avp(avp!(415, None, M, Unsigned32::new(1000)));
    ccr.add_avp(avp!(
        1228,
        Some(10415),
        M,
        Address::new(IPv4(Ipv4Addr::new(127, 0, 0, 1)))
    ));

    let resp = client.send_message(ccr).await.unwrap();
    let cca = resp.await.unwrap();
    log::info!("Received rseponse: {}", cca);
}
