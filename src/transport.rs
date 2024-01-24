use crate::diameter::DiameterMessage;
use crate::error::Error;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub struct Codec {}

impl Codec {
    pub fn new() -> Codec {
        Codec {}
    }

    pub async fn decode<R>(reader: &mut R) -> Result<DiameterMessage, Error>
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

    pub async fn encode<W>(writer: &mut W, msg: &DiameterMessage) -> Result<(), Error>
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
    use crate::avp::enumerated::EnumeratedAvp;
    use crate::avp::identity::IdentityAvp;
    use crate::avp::unsigned32::Unsigned32Avp;
    use crate::avp::utf8string::UTF8StringAvp;
    use crate::avp::Avp;
    use crate::client::DiameterClient;
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage, REQUEST_FLAG};
    use crate::error::Error;
    use crate::server::DiameterServer;

    #[tokio::test]
    async fn test_diameter_transport() {
        // Diameter Server
        let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();

        tokio::spawn(async move {
            server
                .listen(|req| -> Result<DiameterMessage, Error> {
                    println!("Request : {}", req);

                    let mut res = DiameterMessage::new(
                        req.get_command_code(),
                        req.get_application_id(),
                        req.get_flags() ^ REQUEST_FLAG,
                        req.get_hop_by_hop_id(),
                        req.get_end_to_end_id(),
                    );
                    res.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
                    res.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
                    res.add_avp(avp!(263, None, UTF8StringAvp::new("ses;123458890"), true));
                    res.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
                    res.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
                    res.add_avp(avp!(268, None, Unsigned32Avp::new(2001), true));
                    Ok(res)
                })
                .await
                .unwrap();
        });

        // Diameter Client
        let mut client = DiameterClient::new("localhost:3868");
        let _ = client.connect().await;

        let mut ccr = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            REQUEST_FLAG,
            1123158611,
            3102381851,
        );
        ccr.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
        ccr.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
        ccr.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345888"), true));
        ccr.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
        ccr.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
        let cca = client.send_message(ccr).await.unwrap();

        println!("Response: {}", cca);

        // Assert Result-Code
        let result_code = &cca.get_avp(268).unwrap();
        assert_eq!(result_code.get_unsigned32().unwrap(), 2001);
    }
}
