//! Diameter Protocol Transport
use crate::diameter::DiameterMessage;
use crate::error::{Error, Result};
use std::io::Cursor;
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
    pub async fn decode<R>(reader: &mut R) -> Result<DiameterMessage>
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
        DiameterMessage::decode_from(&mut cursor)
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
    use crate::avp::identity::Identity;
    use crate::avp::unsigned32::Unsigned32;
    use crate::avp::utf8string::UTF8String;
    use crate::avp::Avp;
    use crate::avp::Unsigned64;
    use crate::client::DiameterClient;
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage, REQUEST_FLAG};
    use crate::error::Result;
    use crate::server::DiameterServer;

    #[tokio::test]
    async fn test_diameter_transport() {
        // Diameter Server
        let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();

        tokio::spawn(async move {
            server
                .listen(|req| -> Result<DiameterMessage> {
                    println!("Request : {}", req);

                    let mut res = DiameterMessage::new(
                        req.get_command_code(),
                        req.get_application_id(),
                        req.get_flags() ^ REQUEST_FLAG,
                        req.get_hop_by_hop_id(),
                        req.get_end_to_end_id(),
                    );
                    res.add_avp(avp!(264, None, Identity::new("host.example.com"), true));
                    res.add_avp(avp!(296, None, Identity::new("realm.example.com"), true));
                    res.add_avp(avp!(263, None, UTF8String::new("ses;123458890"), true));
                    res.add_avp(avp!(416, None, Enumerated::new(1), true));
                    res.add_avp(avp!(415, None, Unsigned32::new(1000), true));
                    res.add_avp(avp!(268, None, Unsigned32::new(2001), true));
                    Ok(res)
                })
                .await
                .unwrap();
        });

        // Diameter Client
        let mut client = DiameterClient::new("localhost:3868");
        let _ = client.connect().await;

        // Send Single CCR
        let mut ccr = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            REQUEST_FLAG,
            1123158611,
            3102381851,
        );
        ccr.add_avp(avp!(264, None, Identity::new("host.example.com"), true));
        ccr.add_avp(avp!(296, None, Identity::new("realm.example.com"), true));
        ccr.add_avp(avp!(263, None, UTF8String::new("ses;12345888"), true));
        ccr.add_avp(avp!(416, None, Enumerated::new(1), true));
        ccr.add_avp(avp!(415, None, Unsigned32::new(1000), true));
        let cca = client.send_message(ccr).await.unwrap();

        println!("Response: {}", cca);

        // Assert Result-Code
        let result_code = &cca.get_avp(268).unwrap();
        assert_eq!(result_code.get_unsigned32().unwrap(), 2001);

        // Send Multiple CCRs
        let mut handles = vec![];
        let n = 3;
        let mut seq_no = 0;

        for _ in 0..n {
            seq_no = seq_no + 1;
            let mut ccr = DiameterMessage::new(
                CommandCode::CreditControl,
                ApplicationId::CreditControl,
                REQUEST_FLAG,
                seq_no,
                seq_no,
            );
            ccr.add_avp(avp!(264, None, Identity::new("host.example.com"), true));
            ccr.add_avp(avp!(296, None, Identity::new("realm.example.com"), true));
            ccr.add_avp(avp!(263, None, UTF8String::new("ses;12345888"), true));
            ccr.add_avp(avp!(416, None, Enumerated::new(1), true));
            ccr.add_avp(avp!(415, None, Unsigned64::new(1000), true));
            let mut request = client.request(ccr).await.unwrap();
            let handle = tokio::spawn(async move {
                let _ = request.send().await.unwrap();
                let cca = request.response().await.unwrap();

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
