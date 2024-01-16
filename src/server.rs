use crate::diameter::DiameterMessage;
use crate::error::Error;
use log::error;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

pub struct DiameterServer {
    listener: TcpListener,
}

impl DiameterServer {
    pub async fn new(addr: &str) -> Result<DiameterServer, Error> {
        let listener = TcpListener::bind(addr).await?;
        Ok(DiameterServer { listener })
    }

    pub async fn handle<F>(&mut self, handler: F) -> Result<(), Error>
    where
        F: Fn(DiameterMessage) -> Result<DiameterMessage, Error> + Clone + Send + 'static,
    {
        loop {
            let (mut socket, _) = self.listener.accept().await?;
            let handler = handler.clone();
            tokio::spawn(async move {
                let peer_addr = match socket.peer_addr() {
                    Ok(addr) => addr.to_string(),
                    Err(_) => "Unknown".to_string(),
                };

                loop {
                    // Read first 4 bytes to determine the length
                    let mut b = [0; 4];
                    if let Err(e) = socket.read_exact(&mut b).await {
                        error!(
                            "Failed to read header from socket (client: {}); error: {:?}",
                            peer_addr, e
                        );
                        return;
                    }
                    let length = u32::from_be_bytes([0, b[1], b[2], b[3]]);

                    // Limit to 1MB
                    if length > 1024 * 1024 {
                        error!("Message too large (client: {})", peer_addr);
                        return;
                    }

                    // Read the rest of the message
                    let mut buf = vec![0; length as usize - 4];
                    if let Err(e) = socket.read_exact(&mut buf).await {
                        error!(
                            "Failed to read message from socket (client: {}); error: {:?}",
                            peer_addr, e
                        );
                        return;
                    }

                    let mut request = Vec::with_capacity(length as usize);
                    request.extend_from_slice(&b);
                    request.append(&mut buf);

                    // Decode the request
                    let mut cursor = Cursor::new(request);
                    let req = match DiameterMessage::decode_from(&mut cursor) {
                        Ok(req) => req,
                        Err(e) => {
                            error!("failed to decode request; err = {:?}", e);
                            return;
                        }
                    };

                    // Process the request using the handler
                    let res = match handler(req) {
                        Ok(res) => res,
                        Err(e) => {
                            error!("request handler error: {:?}", e);
                            return;
                        }
                    };

                    // Encode and send the response
                    let mut response = Vec::new();
                    if res.encode_to(&mut response).is_err() {
                        error!("failed to encode response");
                        return;
                    }

                    // Send the response
                    if let Err(e) = socket.write_all(&response).await {
                        error!("failed to write to socket; err = {:?}", e);
                        return;
                    }
                }
            });
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
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage};

    #[ignore]
    #[tokio::test]
    async fn test_server() {
        let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();
        server
            .handle(|_req| -> Result<DiameterMessage, Error> {
                // Return Dummy Value
                let mut res = DiameterMessage::new(
                    CommandCode::CreditControl,
                    ApplicationId::CreditControl,
                    0,
                    1123158610,
                    3102381851,
                );
                res.add_avp(avp!(264, None, IdentityAvp::new("host.example.com"), true));
                res.add_avp(avp!(296, None, IdentityAvp::new("realm.example.com"), true));
                res.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345889"), true));
                res.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
                res.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
                res.add_avp(avp!(268, None, Unsigned32Avp::new(2001), true));
                Ok(res)
            })
            .await
            .unwrap();
    }
}
