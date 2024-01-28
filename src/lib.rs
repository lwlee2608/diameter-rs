//! # Diameter
//!
//! Rust Implementation of the Diameter Protocol.
//!
//! ## Reference
//! Based on [RFC 6733](https://tools.ietf.org/html/rfc6733)
//!
//! ## Usage
//!
//! ### Diameter Server Example
//! ```no_run
//! use diameter::avp;
//! use diameter::avp::Avp;
//! use diameter::avp::Enumerated;
//! use diameter::avp::Identity;
//! use diameter::avp::Unsigned32;
//! use diameter::avp::UTF8String;
//! use diameter::Result;
//! use diameter::DiameterServer;
//! use diameter::DiameterMessage;
//! use diameter::REQUEST_FLAG;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Set up a Diameter server listening on a specific port
//!     let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();
//!
//!     // Asynchronously handle incoming requests to the server
//!     server.listen(|req| -> Result<DiameterMessage> {
//!         println!("Received request: {}", req);
//!
//!         // Create a response message based on the received request
//!         let mut res = DiameterMessage::new(
//!             req.get_command_code(),
//!             req.get_application_id(),
//!             req.get_flags() ^ REQUEST_FLAG,
//!             req.get_hop_by_hop_id(),
//!             req.get_end_to_end_id(),
//!         );
//!
//!         // Add various Attribute-Value Pairs (AVPs) to the response
//!         res.add_avp(avp!(264, None, Identity::new("host.example.com"), true));
//!         res.add_avp(avp!(296, None, Identity::new("realm.example.com"), true));
//!         res.add_avp(avp!(263, None, UTF8String::new("ses;123458890"), true));
//!         res.add_avp(avp!(416, None, Enumerated::new(1), true));
//!         res.add_avp(avp!(415, None, Unsigned32::new(1000), true));
//!         res.add_avp(avp!(268, None, Unsigned32::new(2001), true));
//!
//!         // Return the response
//!         Ok(res)
//!     }).await.unwrap();
//! }
//! ```
//!
//! ### Diameter Client Example
//! ```no_run
//! use diameter::avp;
//! use diameter::avp::Avp;
//! use diameter::avp::Identity;
//! use diameter::avp::Enumerated;
//! use diameter::avp::Unsigned32;
//! use diameter::avp::UTF8String;
//! use diameter::DiameterClient;
//! use diameter::{ApplicationId, CommandCode, DiameterMessage, REQUEST_FLAG};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Initialize a Diameter client and connect it to the server
//!     let mut client = DiameterClient::new("localhost:3868");
//!     let _ = client.connect().await;
//!
//!     // Create a Credit-Control-Request (CCR) Diameter message
//!     let mut ccr = DiameterMessage::new(
//!         CommandCode::CreditControl,
//!         ApplicationId::CreditControl,
//!         REQUEST_FLAG,
//!         1123158611,
//!         3102381851,
//!     );
//!     ccr.add_avp(avp!(264, None, Identity::new("host.example.com"), true));
//!     ccr.add_avp(avp!(296, None, Identity::new("realm.example.com"), true));
//!     ccr.add_avp(avp!(263, None, UTF8String::new("ses;12345888"), true));
//!     ccr.add_avp(avp!(416, None, Enumerated::new(1), true));
//!     ccr.add_avp(avp!(415, None, Unsigned32::new(1000), true));
//!
//!     // Send the CCR message to the server and wait for a response
//!     let cca = client.send_message(ccr).await.unwrap();
//!     println!("Received response: {}", cca);
//! }
//! ```
pub mod avp;
pub mod client;
pub mod diameter;
pub mod dictionary;
pub mod error;
pub mod server;
pub mod transport;

// pub use crate::avp::{Avp, AvpFlags, AvpHeader, AvpType, AvpValue};
pub use crate::client::DiameterClient;
pub use crate::diameter::{ApplicationId, CommandCode, DiameterHeader, DiameterMessage};
pub use crate::diameter::{
    ERROR_FLAG, HEADER_LENGTH, PROXYABLE_FLAG, REQUEST_FLAG, RETRANSMIT_FLAG,
};
pub use crate::error::{Error, Result};
pub use crate::server::DiameterServer;
