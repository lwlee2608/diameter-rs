use crate::diameter::DiameterMessage;
use crate::error::Error;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub struct DiameterClient {
    stream: Option<TcpStream>,
}

impl DiameterClient {
    pub fn new() -> DiameterClient {
        DiameterClient { stream: None }
    }

    pub async fn connect(&mut self, addr: &str) -> Result<(), Error> {
        let stream = TcpStream::connect(addr).await?;
        self.stream = Some(stream);
        Ok(())
    }

    pub async fn send(&mut self, req: DiameterMessage) -> Result<DiameterMessage, Error> {
        if let Some(stream) = self.stream.as_mut() {
            // Encode Request
            let mut encoded = Vec::new();
            req.encode_to(&mut encoded)?;

            // Send Request
            stream.write_all(&encoded).await?;

            // Read first 4 bytes to determine the length
            let mut b = [0; 4];
            stream.read_exact(&mut b).await?;
            let length = u32::from_be_bytes([0, b[1], b[2], b[3]]);

            // Limit to 1MB
            if length as usize > 1024 * 1024 {
                return Err(Error::ClientError("Message too large ".into()));
            }

            // Read the rest of the message
            let mut buffer = Vec::with_capacity(length as usize);
            buffer.extend_from_slice(&b);
            buffer.resize(length as usize, 0);
            stream.read_exact(&mut buffer[4..]).await?;

            // Decode Response
            let mut cursor = Cursor::new(buffer);
            let res = DiameterMessage::decode_from(&mut cursor)?;

            Ok(res)
        } else {
            Err(Error::ClientError("Not connected".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::avp;
    use crate::avp::enumerated::EnumeratedAvp;
    use crate::avp::identity::IdentityAvp;
    use crate::avp::unsigned32::Unsigned32Avp;
    use crate::avp::utf8string::UTF8StringAvp;
    use crate::avp::Avp;
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage, REQUEST_FLAG};

    #[ignore]
    #[tokio::test]
    async fn test_diameter_client() {
        let mut ccr = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            REQUEST_FLAG,
            1123158610,
            3102381851,
        );
        ccr.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
        ccr.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
        ccr.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345888"), true));
        ccr.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
        ccr.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));

        let mut client = DiameterClient::new();
        let _ = client.connect("localhost:3868").await;
        let response = client.send(ccr).await.unwrap();
        println!("Response: {}", response);
    }
}
