use chrono::Local;
use diameter::avp::flags::M;
use diameter::avp::Enumerated;
use diameter::avp::Grouped;
use diameter::avp::Identity;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::dictionary::{self, Dictionary};
use diameter::flags;
use diameter::transport::DiameterServer;
use diameter::transport::DiameterServerConfig;
use diameter::CommandCode;
use diameter::DiameterMessage;
use std::fs;
use std::io::Write;
use std::sync::Arc;
use std::thread;

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
    let dict = Dictionary::new(&[
        &dictionary::DEFAULT_DICT_XML,
        &fs::read_to_string("dict/3gpp-ro-rf.xml").unwrap(),
    ]);
    let dict = Arc::new(dict);

    let config = DiameterServerConfig { native_tls: None };

    // Set up a Diameter server listening on a specific port
    let addr = "0.0.0.0:3868";
    let mut server = DiameterServer::new(addr, config).await.unwrap();
    log::info!("Listening at {}", addr);

    // Asynchronously handle incoming requests to the server
    let dict_ref = Arc::clone(&dict);
    server
        .listen(
            move |req| {
                let dict_ref2 = Arc::clone(&dict);
                async move {
                    log::info!("Received request: {}", req);

                    // Create a response message based on the received request
                    let mut res = DiameterMessage::new(
                        req.get_command_code(),
                        req.get_application_id(),
                        req.get_flags() ^ flags::REQUEST,
                        req.get_hop_by_hop_id(),
                        req.get_end_to_end_id(),
                        Arc::clone(&dict_ref2),
                    );

                    match req.get_command_code() {
                        CommandCode::CapabilitiesExchange => {
                            res.add_avp(264, None, M, Identity::new("host.example.com").into());
                            res.add_avp(296, None, M, Identity::new("realm.example.com").into());
                            res.add_avp(266, None, M, Unsigned32::new(35838).into());
                            res.add_avp(269, None, M, UTF8String::new("diameter-rs").into());
                            res.add_avp(258, None, M, Unsigned32::new(4).into());
                            res.add_avp(268, None, M, Unsigned32::new(2001).into());
                        }
                        _ => {
                            res.add_avp(264, None, M, Identity::new("host.example.com").into());
                            res.add_avp(296, None, M, Identity::new("realm.example.com").into());
                            res.add_avp(263, None, M, UTF8String::new("ses;123458890").into());
                            res.add_avp(416, None, M, Enumerated::new(1).into());
                            res.add_avp(415, None, M, Unsigned32::new(1000).into());
                            res.add_avp(268, None, M, Unsigned32::new(2001).into());

                            let mut mscc = Grouped::new(vec![], Arc::clone(&dict_ref2));
                            mscc.add_avp(439, None, M, Unsigned32::new(7786).into());
                            mscc.add_avp(432, None, M, Unsigned32::new(7786).into());
                            mscc.add_avp(268, None, M, Unsigned32::new(2001).into());
                            res.add_avp(456, None, M, mscc.into());

                            let mut ps_info = Grouped::new(vec![], Arc::clone(&dict_ref2));
                            ps_info.add_avp(30, None, M, UTF8String::new("10999").into());
                            let mut service_info = Grouped::new(vec![], Arc::clone(&dict_ref2));
                            service_info.add_avp(874, Some(10415), M, ps_info.into());
                            res.add_avp(873, Some(10415), M, service_info.into());
                        }
                    }

                    // Simulate a delay
                    // tokio::time::sleep(std::time::Duration::from_millis(100)).await;

                    // Return the response
                    Ok(res)
                }
            },
            dict_ref,
        )
        .await
        .unwrap();
}
