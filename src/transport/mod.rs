//! Diameter Protocol Transport

pub mod client;
pub mod server;

use crate::dictionary::Dictionary;
pub use crate::transport::client::DiameterClient;
pub use crate::transport::client::DiameterClientConfig;
pub use crate::transport::server::DiameterServer;
pub use crate::transport::server::DiameterServerConfig;

use crate::diameter::DiameterMessage;
use crate::error::{Error, Result};
use std::io::Cursor;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// Codec provides encoding and decoding functionality for Diameter messages
/// over the TCP transport layer.
pub struct Codec {}

impl Codec {
    /// Asynchronously decodes a DiameterMessage from a reader.
    ///
    /// Reads from `reader`, decodes according to Diameter protocol standards, and returns a DiameterMessage.
    ///
    /// # Arguments
    /// * `reader` - A mutable reference to an object implementing `AsyncReadExt` and `Unpin`.
    pub async fn decode<R>(reader: &mut R, dict: Arc<Dictionary>) -> Result<DiameterMessage>
    where
        R: AsyncReadExt + Unpin,
    {
        let mut b = [0; 4];
        reader.read_exact(&mut b).await?;
        let length = u32::from_be_bytes([0, b[1], b[2], b[3]]);

        // Limit to 1MB
        if length as usize > 1024 * 1024 {
            return Err(Error::ClientError("Message too large to read".into()));
        }

        // Read the rest of the message
        let mut buffer = Vec::with_capacity(length as usize);
        buffer.extend_from_slice(&b);
        buffer.resize(length as usize, 0);
        reader.read_exact(&mut buffer[4..]).await?;

        // Decode Response
        let mut cursor = Cursor::new(buffer);
        DiameterMessage::decode_from(&mut cursor, dict)
    }

    /// Asynchronously encodes a DiameterMessage and writes it to a writer.
    ///
    /// Encodes DiameterMessage into a byte stream and writes to `writer`.
    ///
    /// # Arguments
    /// * `writer` - A mutable reference to an object implementing `AsyncWriteExt` and `Unpin`.
    /// * `msg` - A reference to the `DiameterMessage` to encode.
    pub async fn encode<W>(writer: &mut W, msg: &DiameterMessage) -> Result<()>
    where
        W: AsyncWriteExt + Unpin,
    {
        // Encode and send the response
        let mut b = Vec::new();
        msg.encode_to(&mut b)?;

        // Send the response
        writer.write_all(&b).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::avp;
    use crate::avp::enumerated::Enumerated;
    use crate::avp::flags::M;
    use crate::avp::identity::Identity;
    use crate::avp::unsigned32::Unsigned32;
    use crate::avp::utf8string::UTF8String;
    use crate::avp::Avp;
    use crate::avp::Unsigned64;
    use crate::diameter::flags;
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage};
    use crate::dictionary;
    use crate::dictionary::Dictionary;
    use crate::transport::DiameterClient;
    use crate::transport::DiameterClientConfig;
    use crate::transport::DiameterServer;
    use crate::transport::DiameterServerConfig;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_diameter_transport() {
        // Dictionary
        let dict = Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);

        // Diameter Server
        let mut server =
            DiameterServer::new("0.0.0.0:3868", DiameterServerConfig { native_tls: None })
                .await
                .unwrap();

        let dict_ref = Arc::new(dict.clone());
        tokio::spawn(async move {
            let dict_ref2 = Arc::clone(&dict_ref);
            server
                .listen(
                    move |req| {
                        let dict_ref2 = Arc::clone(&dict_ref2);
                        async move {
                            println!("Request : {}", req);

                            let mut res = DiameterMessage::new(
                                req.get_command_code(),
                                req.get_application_id(),
                                req.get_flags() ^ flags::REQUEST,
                                req.get_hop_by_hop_id(),
                                req.get_end_to_end_id(),
                                dict_ref2,
                            );
                            res.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
                            res.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
                            res.add_avp(avp!(263, None, M, UTF8String::new("ses;123458890")));
                            res.add_avp(avp!(416, None, M, Enumerated::new(1)));
                            res.add_avp(avp!(415, None, M, Unsigned32::new(1000)));
                            res.add_avp(avp!(268, None, M, Unsigned32::new(2001)));
                            Ok(res)
                        }
                    },
                    dict_ref,
                )
                .await
                .unwrap();
        });

        // Diameter Client
        let client_config = DiameterClientConfig {
            use_tls: false,
            verify_cert: false,
        };
        let mut client = DiameterClient::new("localhost:3868", client_config);
        let mut handler = client.connect().await.unwrap();
        let dict_ref = Arc::new(dict.clone());
        tokio::spawn(async move {
            DiameterClient::handle(&mut handler, dict_ref).await;
        });

        // Send Single CCR
        let dict_ref = Arc::new(dict.clone());
        let mut ccr = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            flags::REQUEST,
            1123158611,
            3102381851,
            Arc::clone(&dict_ref),
        );
        ccr.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
        ccr.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
        ccr.add_avp(avp!(263, None, M, UTF8String::new("ses;12345888")));
        ccr.add_avp(avp!(416, None, M, Enumerated::new(1)));
        ccr.add_avp(avp!(415, None, M, Unsigned32::new(1000)));
        // let cca = client.send_message(ccr).await.unwrap();
        let response = client.send_message(ccr).await.unwrap();
        let cca = response.await.unwrap();

        println!("Response: {}", cca);

        // Assert Result-Code
        let result_code = &cca.get_avp(268).unwrap();
        assert_eq!(result_code.get_unsigned32().unwrap(), 2001);

        // Send Multiple CCRs
        let mut handles = vec![];
        let n = 3;

        for _ in 0..n {
            let seq_num = client.get_next_seq_num();
            let mut ccr = DiameterMessage::new(
                CommandCode::CreditControl,
                ApplicationId::CreditControl,
                flags::REQUEST,
                seq_num,
                seq_num,
                Arc::clone(&dict_ref),
            );
            ccr.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
            ccr.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
            ccr.add_avp(avp!(263, None, M, UTF8String::new("ses;12345888")));
            ccr.add_avp(avp!(416, None, M, Enumerated::new(1)));
            ccr.add_avp(avp!(415, None, M, Unsigned64::new(1000)));
            let response = client.send_message(ccr).await.unwrap();
            let handle = tokio::spawn(async move {
                let cca = response.await.unwrap();
                println!("Response: {}", cca);

                // Assert Result-Code
                let result_code = &cca.get_avp(268).unwrap();
                assert_eq!(result_code.get_unsigned32().unwrap(), 2001);
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }
}
