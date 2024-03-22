use chrono::Local;
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
use diameter::flags;
use diameter::transport::eventloop::DiameterClient;
use diameter::{ApplicationId, CommandCode, DiameterMessage};
use std::fs;
use std::io::Write;
use std::net::Ipv4Addr;
use std::thread;
use tokio::task;
use tokio::task::JoinHandle;
use tokio::task::LocalSet;

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .format(|buf, record| {
            let now = Local::now();
            let thread = thread::current();
            let thread_name = thread.name().unwrap_or("unnamed");
            let thread_id = thread.id();

            writeln!(
                buf,
                "{} [{}] {:?} - ({}): {}",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                thread_id,
                thread_name,
                record.args()
            )
        })
        .filter(None, log::LevelFilter::Info)
        .init();

    // Load dictionary
    {
        let mut dictionary = dictionary::DEFAULT_DICT.write().unwrap();
        let xml = fs::read_to_string("dict/3gpp-ro-rf.xml").unwrap();
        dictionary.load_xml(&xml);
    }

    let local = LocalSet::new();
    local
        .run_until(async move {
            // Initialize a Diameter client and connect it to the server
            let mut client = DiameterClient::new("localhost:3868");
            let _ = client.connect().await;

            // Send a Capabilities-Exchange-Request (CER) Diameter message
            send_cer(&mut client).await;

            // Send a batch of Credit-Control-Request (CCR) Diameter message
            let mut handles = vec![];
            let batch_size = 10;
            for _ in 0..batch_size {
                let handle = send_ccr(&mut client).await;
                handles.push(handle);
            }

            for handle in handles {
                handle.await.unwrap();
            }
        })
        .await
}

async fn send_cer(client: &mut DiameterClient) {
    let seq_num = client.get_next_seq_num();
    let mut cer = DiameterMessage::new(
        CommandCode::CapabilitiesExchange,
        ApplicationId::Common,
        flags::REQUEST,
        seq_num,
        seq_num,
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

    let cea = client.send_message(cer).await.unwrap();
    log::info!("Received rseponse: {}", cea);
}

async fn send_ccr(client: &mut DiameterClient) -> JoinHandle<()> {
    let seq_num = client.get_next_seq_num();
    let mut ccr = DiameterMessage::new(
        CommandCode::CreditControl,
        ApplicationId::CreditControl,
        flags::REQUEST,
        seq_num,
        seq_num,
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

    let mut request = client.request(ccr).await.unwrap();
    log::info!("Request sent id: {}", seq_num);

    let handle = task::spawn_local(async move {
        let _ = request.send().await.unwrap();
        let cca = request.response().await.unwrap();
        let seq_num = cca.get_hop_by_hop_id();
        log::info!("Response recv id: {}", seq_num);
    });

    handle
}
