//! Diameter Protocol Client
use crate::diameter::DiameterMessage;
use crate::error::{Error, Result};
use crate::transport::Codec;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;
use tokio::task;

/// A Diameter protocol client for sending and receiving Diameter messages.
///
/// The client maintains a connection to a Diameter server and provides
/// functionality for sending requests and asynchronously receiving responses.
///
/// Fields:
///     address: The address of the Diameter server to connect to.
///     writer: An optional thread-safe writer for sending messages to the server.
///     msg_caches: A shared, mutable hash map that maps message IDs to channels for sending responses back to the caller.
///     seq_num: The next sequence number to use for a message.

pub struct DiameterClient {
    address: String,
    writer: Option<Arc<Mutex<OwnedWriteHalf>>>,
    msg_caches: Arc<Mutex<HashMap<u32, Sender<DiameterMessage>>>>,
    seq_num: u32,
}

impl DiameterClient {
    pub fn new(addr: &str) -> DiameterClient {
        DiameterClient {
            address: addr.into(),
            writer: None,
            msg_caches: Arc::new(Mutex::new(HashMap::new())),
            seq_num: 0,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        let stream = TcpStream::connect(self.address.clone()).await?;

        let (mut reader, writer) = stream.into_split();
        let writer = Arc::new(Mutex::new(writer));

        self.writer = Some(writer);

        let msg_caches = Arc::clone(&self.msg_caches);
        task::spawn_local(async move {
            loop {
                match Codec::decode(&mut reader).await {
                    Ok(res) => {
                        if let Err(e) = Self::process_decoded_msg(msg_caches.clone(), res).await {
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

    async fn process_decoded_msg(
        msg_caches: Arc<Mutex<HashMap<u32, Sender<DiameterMessage>>>>,
        res: DiameterMessage,
    ) -> Result<()> {
        let hop_by_hop = res.get_hop_by_hop_id();

        let sender_opt = {
            let mut msg_caches = msg_caches.lock().await;

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

    pub async fn request(&mut self, req: DiameterMessage) -> Result<DiameterRequest> {
        if let Some(writer) = &self.writer {
            let (tx, rx) = oneshot::channel();
            let hop_by_hop = req.get_hop_by_hop_id();
            {
                let mut msg_caches = self.msg_caches.lock().await;
                msg_caches.insert(hop_by_hop, tx);
            }

            Ok(DiameterRequest::new(req, rx, Arc::clone(&writer)))
        } else {
            Err(Error::ClientError("Not connected".into()))
        }
    }

    pub async fn send_message(&mut self, req: DiameterMessage) -> Result<DiameterMessage> {
        let mut request = self.request(req).await?;
        let _ = request.send().await?;
        let response = request.response().await?;
        Ok(response)
    }

    // Returns the next sequence number.
    pub fn get_next_seq_num(&mut self) -> u32 {
        self.seq_num += 1;
        self.seq_num
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

    pub async fn send(&mut self) -> Result<()> {
        let mut writer = self.writer.lock().await;
        Codec::encode(&mut writer.deref_mut(), &self.request).await
    }

    pub async fn response(&self) -> Result<DiameterMessage> {
        let rx = self
            .receiver
            .lock()
            .await
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
    use crate::avp::enumerated::Enumerated;
    use crate::avp::flags::M;
    use crate::avp::identity::Identity;
    use crate::avp::unsigned32::Unsigned32;
    use crate::avp::utf8string::UTF8String;
    use crate::avp::Avp;
    use crate::diameter::flags;
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage};

    #[ignore]
    #[tokio::test]
    async fn test_diameter_client() {
        let mut ccr = DiameterMessage::new(
            CommandCode::CreditControl,
            ApplicationId::CreditControl,
            flags::REQUEST,
            1123158610,
            3102381851,
        );
        ccr.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
        ccr.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
        ccr.add_avp(avp!(263, None, M, UTF8String::new("ses;12345888")));
        ccr.add_avp(avp!(416, None, M, Enumerated::new(1)));
        ccr.add_avp(avp!(415, None, M, Unsigned32::new(1000)));

        let mut client = DiameterClient::new("localhost:3868");
        let _ = client.connect().await;
        let response = client.send_message(ccr).await.unwrap();
        println!("Response: {}", response);
    }
}
