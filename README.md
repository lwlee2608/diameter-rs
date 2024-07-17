# Diameter

Rust Implementation of the Diameter Protocol.

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/diameter.svg
[crates-url]: https://crates.io/crates/diameter
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[actions-badge]: https://github.com/lwlee2608/diameter-rs/actions/workflows/build.yml/badge.svg
[actions-url]: https://github.com/lwlee2608/diameter-rs/actions?query=branch%3Amaster

## Overview

This library provides a Rust implementation of the Diameter protocol, as defined by [RFC 6733](https://tools.ietf.org/html/rfc6733).



## Getting Started

### Installation
Add this crate to your Rust project by adding the following to your `Cargo.toml`:

```toml
[dependencies]
diameter-rs = "^0.6"
```


## Usage

### Diameter Server Example
Below is an example of setting up a Diameter server that listens for incoming requests

```rust
use diameter::avp;
use diameter::avp::Avp;
use diameter::avp::Enumerated;
use diameter::avp::Identity;
use diameter::avp::Unsigned32;
use diameter::avp::UTF8String;
use diameter::avp::flags::M;
use diameter::Result;
use diameter::transport::DiameterServer;
use diameter::DiameterMessage;
use diameter::flags;

#[tokio::main]
async fn main() {
    // Diameter Dictionary
    let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
    let dict = Arc::new(dict);

    // Set up a Diameter server listening on a specific port
    let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();

    // Asynchronously handle incoming requests to the server
    let dict_ref = Arc::clone(&dict);
    server
        .listen(
            move |req| {
                let dict_ref2 = Arc::clone(&dict);
                async move {
                    println!("Received request: {}", req);

                    // Create a response message based on the received request
                    let mut res = DiameterMessage::new(
                        req.get_command_code(),
                        req.get_application_id(),
                        req.get_flags() ^ flags::REQUEST,
                        req.get_hop_by_hop_id(),
                        req.get_end_to_end_id(),
                        dict_ref2,
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
                }
            },
            dict_ref,
        )
        .await
        .unwrap();
}

```

### Diameter Client Example
Below is an example of creating a Diameter client that sends a Credit-Control-Request (CCR) message to a server and waits for a response.

```rust
use diameter::avp;
use diameter::avp::Avp;
use diameter::avp::Identity;
use diameter::avp::Enumerated;
use diameter::avp::Unsigned32;
use diameter::avp::UTF8String;
use diameter::avp::flags::M;
use diameter::transport::DiameterClient;
use diameter::{ApplicationId, CommandCode, DiameterMessage};
use diameter::dictionary::{self, Dictionary};
use diameter::flags;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Diameter Dictionary
    let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
    let dict = Arc::new(dict);

    // Initialize a Diameter client and connect it to the server
    let mut client = DiameterClient::new("localhost:3868");
    let mut handler = client.connect().await.unwrap();
    let dict_ref = Arc::clone(&dict);
    tokio::spawn(async move {
        DiameterClient::handle(&mut handler, dict_ref).await;
    });

    // Create a Credit-Control-Request (CCR) Diameter message
    let mut ccr = DiameterMessage::new(
        CommandCode::CreditControl,
        ApplicationId::CreditControl,
        flags::REQUEST,
        1123158611,
        3102381851,
        dict,
    );
    ccr.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
    ccr.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
    ccr.add_avp(avp!(263, None, M, UTF8String::new("ses;12345888")));
    ccr.add_avp(avp!(416, None, M, Enumerated::new(1)));
    ccr.add_avp(avp!(415, None, M, Unsigned32::new(1000)));

    // Send the CCR message to the server and wait for a response
    let response = client.send_message(ccr).await.unwrap();
    let cca = response.await.unwrap();
    println!("Received response: {}", cca);
}
```


## TLS

Below are examples of how to set up TLS for both the server and the client.

### Server Configuration with TLS
```rust
    let mut cert_file = File::open("server.crt").unwrap();
    let mut certs = vec![];
    cert_file.read_to_end(&mut certs).unwrap();

    let mut key_file = File::open("server.key").unwrap();
    let mut key = vec![];
    key_file.read_to_end(&mut key).unwrap();

    let pkcs8 = native_tls::Identity::from_pkcs8(&certs, &key).unwrap();
    let config = DiameterServerConfig {
        native_tls: Some(pkcs8),
    };
```

### Client Configuration with TLS
```rust
    let client_config = DiameterClientConfig {
        use_tls: true,
        verify_cert: false,
    };
```
