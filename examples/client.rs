use diameter::avp;
use diameter::avp::flags::M;
use diameter::avp::Avp;
use diameter::avp::Enumerated;
use diameter::avp::Identity;
use diameter::avp::UTF8String;
use diameter::avp::Unsigned32;
use diameter::flags;
use diameter::transport::DiameterClient;
use diameter::{ApplicationId, CommandCode, DiameterMessage};

#[tokio::main]
async fn main() {
    // Initialize a Diameter client and connect it to the server
    let mut client = DiameterClient::new("localhost:3868");
    let _ = client.connect().await;

    // Create a Credit-Control-Request (CCR) Diameter message
    let mut ccr = DiameterMessage::new(
        CommandCode::CreditControl,
        ApplicationId::CreditControl,
        flags::REQUEST,
        1123158611,
        3102381851,
    );
    ccr.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
    ccr.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
    ccr.add_avp(avp!(263, None, M, UTF8String::new("ses;12345888")));
    ccr.add_avp(avp!(416, None, M, Enumerated::new(1)));
    ccr.add_avp(avp!(415, None, M, Unsigned32::new(1000)));

    // Send the CCR message to the server and wait for a response
    let cca = client.send_message(ccr).await.unwrap();
    println!("Received response: {}", cca);
}
