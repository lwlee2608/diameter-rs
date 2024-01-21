//!Rust Implementation of the Diameter Protocol.
//!
//!# Reference
//!Based on [RFC 6733](https://tools.ietf.org/html/rfc6733)
//!
pub mod avp;
pub mod client;
pub mod diameter;
pub mod dictionary;
pub mod error;
pub mod server;
pub mod transport;
