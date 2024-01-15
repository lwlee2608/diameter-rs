use crate::diameter::DiameterMessage;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

pub struct DiameterServer {
    listener: TcpListener,
}

impl DiameterServer {
    pub async fn new(addr: &str) -> Result<DiameterServer, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        Ok(DiameterServer { listener })
    }

    // TODO
    // replace eprintln!
    // remove unwrap()
    // replace Box<dyn Error>
    // Handle buf 1024 bytes overflow
    // Handle multiple requests in one buffer
    // Handle partial request in buffer
    pub async fn handle<F>(&mut self, handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(DiameterMessage) -> Result<DiameterMessage, Box<dyn std::error::Error>>
            + Copy
            + Send
            + 'static,
    {
        loop {
            let (mut socket, _) = self.listener.accept().await?;
            let handler = handler;
            tokio::spawn(async move {
                let mut buf = [0; 1024];
                loop {
                    let n = match socket.read(&mut buf).await {
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    // Decode the request
                    let request = &buf[..n];
                    let mut cursor = Cursor::new(request);
                    let req = match DiameterMessage::decode_from(&mut cursor) {
                        Ok(req) => req,
                        Err(e) => {
                            eprintln!("failed to decode request; err = {:?}", e);
                            return;
                        }
                    };

                    // Process the request using the handler
                    let res = match handler(req) {
                        Ok(res) => res,
                        Err(e) => {
                            eprintln!("request handler error: {:?}", e);
                            return;
                        }
                    };

                    // Encode and send the response
                    let mut response = Vec::new();
                    if res.encode_to(&mut response).is_err() {
                        eprintln!("failed to encode response");
                        return;
                    }
                    if let Err(e) = socket.write_all(&response).await {
                        eprintln!("failed to write to socket; err = {:?}", e);
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
            .handle(
                |_req| -> Result<DiameterMessage, Box<dyn std::error::Error>> {
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
                },
            )
            .await
            .unwrap();
    }
}
