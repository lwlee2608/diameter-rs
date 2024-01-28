# Diameter

Rust Implementation of the Diameter Protocol.


## Reference
Based on [RFC 6733](https://tools.ietf.org/html/rfc6733)


## Getting Started

### Installation
Add this crate to your Rust project by adding the following to your `Cargo.toml`:

```toml
[dependencies]
diameter-rs = "^0.1"
```

### Diameter Server Example
```rust
use diameter::avp;
use diameter::avp::Avp;
use diameter::avp::Enumerated;
use diameter::avp::Identity;
use diameter::avp::Unsigned32;
use diameter::avp::UTF8String;
use diameter::avp::flags::M;
use diameter::Result;
use diameter::DiameterServer;
use diameter::DiameterMessage;
use diameter::flags;

#[tokio::main]
async fn main() {
    // Set up a Diameter server listening on a specific port
    let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();

    // Asynchronously handle incoming requests to the server
    server.listen(|req| -> Result<DiameterMessage> {
        println!("Received request: {}", req);

        // Create a response message based on the received request
        let mut res = DiameterMessage::new(
            req.get_command_code(),
            req.get_application_id(),
            req.get_flags() ^ flags::REQUEST,
            req.get_hop_by_hop_id(),
            req.get_end_to_end_id(),
        );

        // Add various Attribute-Value Pairs (AVPs) to the response
        res.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
        res.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
        res.add_avp(avp!(263, None, M, UTF8String::new("ses;123458890")));
        res.add_avp(avp!(416, None, M, Enumerated::new(1)));
        res.add_avp(avp!(415, None, M, Unsigned32::new(1000)));
        res.add_avp(avp!(268, None, M, Unsigned32::new(2001)));

        // Return the response
        Ok(res)
    }).await.unwrap();
}

```

### Diameter Client Example
```rust
use diameter::avp;
use diameter::avp::Avp;
use diameter::avp::Identity;
use diameter::avp::Enumerated;
use diameter::avp::Unsigned32;
use diameter::avp::UTF8String;
use diameter::avp::flags::M;
use diameter::DiameterClient;
use diameter::{ApplicationId, CommandCode, DiameterMessage};
use diameter::flags;

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
```
