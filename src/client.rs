use crate::diameter::DiameterMessage;
use crate::error::Error;
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;

pub struct DiameterClient {
    writer: Option<OwnedWriteHalf>,
    futures: Arc<Mutex<HashMap<u32, oneshot::Sender<DiameterMessage>>>>,
}

impl DiameterClient {
    pub fn new() -> DiameterClient {
        DiameterClient {
            writer: None,
            futures: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&mut self, addr: &str) -> Result<(), Error> {
        let stream = TcpStream::connect(addr).await?;

        let (mut reader, writer) = stream.into_split();
        self.writer = Some(writer);

        let futures = self.futures.clone();
        let _read_task = tokio::spawn(async move {
            loop {
                // TODO handle unwrap
                let res = Self::read(&mut reader).await.unwrap();
                let hop_by_hop = res.get_hop_by_hop_id();

                let sender_opt = {
                    let mut futures = futures.lock().unwrap();
                    futures.remove(&hop_by_hop)
                };
                if let Some(sender) = sender_opt {
                    sender.send(res).unwrap();
                }
            }
        });

        Ok(())
    }

    async fn read(reader: &mut OwnedReadHalf) -> Result<DiameterMessage, Error> {
        let mut b = [0; 4];
        reader.read_exact(&mut b).await?;
        let length = u32::from_be_bytes([0, b[1], b[2], b[3]]);

        // Limit to 1MB
        if length as usize > 1024 * 1024 {
            return Err(Error::ClientError("Message too large ".into()));
        }

        // Read the rest of the message
        let mut buffer = Vec::with_capacity(length as usize);
        buffer.extend_from_slice(&b);
        buffer.resize(length as usize, 0);
        reader.read_exact(&mut buffer[4..]).await?;

        // Decode Response
        let mut cursor = Cursor::new(buffer);
        let res = DiameterMessage::decode_from(&mut cursor)?;
        Ok(res)
    }

    pub async fn request(&mut self, req: DiameterMessage) -> Result<DiameterRequest, Error> {
        if let Some(writer) = self.writer.as_mut() {
            // Encode Request
            let mut encoded = Vec::new();
            req.encode_to(&mut encoded)?;

            // Send Request
            writer.write_all(&encoded).await?;

            // Insert a oneshot channel into futures
            let (tx, rx) = oneshot::channel();
            let hop_by_hop = req.get_hop_by_hop_id();

            {
                let mut futures = self.futures.lock().unwrap();
                futures.insert(hop_by_hop, tx);
            }

            Ok(DiameterRequest::new(req, rx))
        } else {
            Err(Error::ClientError("Not connected".into()))
        }
    }

    pub async fn send(&mut self, req: DiameterMessage) -> Result<DiameterMessage, Error> {
        let request = self.request(req).await?;
        let response = request.get_response().await?;
        Ok(response)
    }
}

#[derive(Debug)]
pub struct DiameterRequest {
    pub request: DiameterMessage,
    // pub response: Receiver<DiameterMessage>,
    // pub response: Arc<Mutex<Receiver<DiameterMessage>>>,
    pub response: Arc<Mutex<Option<Receiver<DiameterMessage>>>>,
}

impl DiameterRequest {
    // pub fn new(request: DiameterMessage, response: Receiver<DiameterMessage>) -> Self {
    //     DiameterRequest { request, response }
    // }
    pub fn new(request: DiameterMessage, response: Receiver<DiameterMessage>) -> Self {
        DiameterRequest {
            request,
            response: Arc::new(Mutex::new(Some(response))),
            // response: Arc::new(Mutex::new(response)),
        }
    }

    pub fn get_request(&self) -> &DiameterMessage {
        &self.request
    }

    pub async fn get_response(&self) -> Result<DiameterMessage, Error> {
        let rx = self
            .response
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| Error::ClientError("Receiver already taken".into()))?;

        match rx.await {
            Ok(response) => Ok(response),
            Err(_) => Err(Error::ClientError("Failed to receive response".into())),
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
