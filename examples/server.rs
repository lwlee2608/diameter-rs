use diameter::avp;
use diameter::avp::flags::M;
use diameter::avp::Avp;
use diameter::avp::Enumerated;
use diameter::avp::Grouped;
use diameter::avp::Identity;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::dictionary;
use diameter::flags;
use diameter::transport::DiameterServer;
use diameter::CommandCode;
use diameter::DiameterMessage;
use diameter::Result;
use std::fs;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Load dictionary
    {
        let mut dictionary = dictionary::DEFAULT_DICT.write().unwrap();
        let xml = fs::read_to_string("dict/3gpp-ro-rf.xml").unwrap();
        dictionary.load_xml(&xml);
    }

    // Set up a Diameter server listening on a specific port
    let addr = "0.0.0.0:3868";
    let mut server = DiameterServer::new(addr).await.unwrap();
    log::info!("Listening at {}", addr);

    // Asynchronously handle incoming requests to the server
    server
        .listen(|req| -> Result<DiameterMessage> {
            log::info!("Received request: {}", req);

            // Create a response message based on the received request
            let mut res = DiameterMessage::new(
                req.get_command_code(),
                req.get_application_id(),
                req.get_flags() ^ flags::REQUEST,
                req.get_hop_by_hop_id(),
                req.get_end_to_end_id(),
            );

            match req.get_command_code() {
                CommandCode::CapabilitiesExchange => {
                    res.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
                    res.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
                    res.add_avp(avp!(266, None, M, Unsigned32::new(35838)));
                    res.add_avp(avp!(269, None, M, UTF8String::new("diameter-rs")));
                    res.add_avp(avp!(258, None, M, Unsigned32::new(4)));
                    res.add_avp(avp!(268, None, M, Unsigned32::new(2001)));
                }
                _ => {
                    res.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
                    res.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
                    res.add_avp(avp!(263, None, M, UTF8String::new("ses;123458890")));
                    res.add_avp(avp!(416, None, M, Enumerated::new(1)));
                    res.add_avp(avp!(415, None, M, Unsigned32::new(1000)));
                    res.add_avp(avp!(268, None, M, Unsigned32::new(2001)));
                    res.add_avp(avp!(
                        456,
                        None,
                        M,
                        Grouped::new(vec![
                            avp!(439, None, M, Unsigned32::new(7786)),
                            avp!(432, None, M, Unsigned32::new(7786)),
                            avp!(268, None, M, Unsigned32::new(2001)),
                        ])
                    ));
                    res.add_avp(avp!(
                        873,
                        Some(10415),
                        M,
                        Grouped::new(vec![avp!(
                            874,
                            Some(10415),
                            M,
                            Grouped::new(vec![avp!(30, None, M, UTF8String::new("10099")),])
                        ),])
                    ));
                }
            }

            // Simulate a delay
            //tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            // Return the response
            Ok(res)
        })
        .await
        .unwrap();
}
