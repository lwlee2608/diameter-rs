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
use tokio::sync::oneshot::Sender;

pub struct DiameterClient {
    writer: Option<Arc<Mutex<OwnedWriteHalf>>>,
    msg_caches: Arc<Mutex<HashMap<u32, Sender<DiameterMessage>>>>,
}

impl DiameterClient {
    pub fn new() -> DiameterClient {
        DiameterClient {
            writer: None,
            msg_caches: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn connect(&mut self, addr: &str) -> Result<(), Error> {
        let stream = TcpStream::connect(addr).await?;

        let (mut reader, writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(writer));
        self.writer = Some(writer);

        let msg_caches = Arc::clone(&self.msg_caches);
        tokio::spawn(async move {
            loop {
                match Self::read(&mut reader).await {
                    Ok(res) => {
                        if let Err(e) = Self::process_response(msg_caches.clone(), res).await {
                            log::error!("Failed to process response; error: {:?}", e);
                            return;
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to read message from socket; error: {:?}", e);
                        return;
                    }
                }
            }
        });

        Ok(())
    }

    async fn process_response(
        msg_caches: Arc<Mutex<HashMap<u32, Sender<DiameterMessage>>>>,
        res: DiameterMessage,
    ) -> Result<(), Error> {
        let hop_by_hop = res.get_hop_by_hop_id();

        let sender_opt = {
            let mut msg_caches = msg_caches.lock()?;

            msg_caches.remove(&hop_by_hop)
        };
        match sender_opt {
            Some(sender) => {
                sender.send(res).map_err(|e| {
                    Error::ClientError(format!("Failed to send response; error: {:?}", e))
                })?;
            }
            None => {
                Err(Error::ClientError(format!(
                    "No request found for hop_by_hop_id {}",
                    hop_by_hop
                )))?;
            }
        };
        Ok(())
    }

    async fn read(reader: &mut OwnedReadHalf) -> Result<DiameterMessage, Error> {
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
        let res = DiameterMessage::decode_from(&mut cursor)?;
        Ok(res)
    }

    pub fn request(&mut self, req: DiameterMessage) -> Result<DiameterRequest, Error> {
        if let Some(writer) = &self.writer {
            let (tx, rx) = oneshot::channel();
            let hop_by_hop = req.get_hop_by_hop_id();
            {
                let mut msg_caches = self.msg_caches.lock()?;
                msg_caches.insert(hop_by_hop, tx);
            }

            Ok(DiameterRequest::new(req, rx, Arc::clone(&writer)))
        } else {
            Err(Error::ClientError("Not connected".into()))
        }
    }

    pub async fn send_message(&mut self, req: DiameterMessage) -> Result<DiameterMessage, Error> {
        let mut request = self.request(req)?;
        let _ = request.send().await?;
        let response = request.response().await?;
        Ok(response)
    }
}

pub struct DiameterRequest {
    request: DiameterMessage,
    receiver: Arc<Mutex<Option<Receiver<DiameterMessage>>>>,
    writer: Arc<Mutex<OwnedWriteHalf>>,
}

impl DiameterRequest {
    pub fn new(
        request: DiameterMessage,
        receiver: Receiver<DiameterMessage>,
        writer: Arc<Mutex<OwnedWriteHalf>>,
    ) -> Self {
        DiameterRequest {
            request,
            receiver: Arc::new(Mutex::new(Some(receiver))),
            writer,
        }
    }

    pub fn get_request(&self) -> &DiameterMessage {
        &self.request
    }

    pub async fn send(&mut self) -> Result<(), Error> {
        let mut encoded = Vec::new();
        self.request.encode_to(&mut encoded)?;

        let mut writer = self.writer.lock()?;
        writer.write_all(&encoded).await?;

        Ok(())
    }

    pub async fn response(&self) -> Result<DiameterMessage, Error> {
        let rx = self
            .receiver
            .lock()?
            .take()
            .ok_or_else(|| Error::ClientError("Response already taken".into()))?;

        let res = rx.await.map_err(|e| {
            Error::ClientError(format!("Failed to receive response; error: {:?}", e))
        })?;

        Ok(res)
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
        let response = client.send_message(ccr).await.unwrap();
        println!("Response: {}", response);
    }
}
