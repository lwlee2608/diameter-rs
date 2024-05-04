//! # Diameter
//!
//! Rust Implementation of the Diameter Protocol.
//!
//! ## Reference
//! Based on [RFC 6733](https://tools.ietf.org/html/rfc6733)
//!
//! ## Examples
//! * [`client`] - A simple diameter client that sends a request to a server and prints the response.
//!
//! * [`server`] - A simple diameter server that listens for requests and sends a response.
//!
//! [`server`]: https://github.com/lwlee2608/diameter-rs/blob/v0.6.0/examples/server.rs
//! [`client`]: https://github.com/lwlee2608/diameter-rs/blob/v0.6.0/examples/client.rs

pub mod avp;
pub mod diameter;
pub mod dictionary;
pub mod error;
pub mod transport;

pub use crate::diameter::flags;
pub use crate::diameter::{ApplicationId, CommandCode, DiameterHeader, DiameterMessage};
pub use crate::error::{Error, Result};
