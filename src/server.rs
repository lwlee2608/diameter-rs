use crate::avp;
use crate::avp::enumerated::EnumeratedAvp;
use crate::avp::identity::IdentityAvp;
use crate::avp::unsigned32::Unsigned32Avp;
use crate::avp::utf8string::UTF8StringAvp;
use crate::avp::Avp;
use crate::diameter::{ApplicationId, CommandCode, DiameterMessage};
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

    pub async fn handle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Accept new connection
            let (mut socket, _) = self.listener.accept().await?;
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

                    // Decode Request
                    let request = &buf[..n];
                    let mut cursor = Cursor::new(request);
                    let req = DiameterMessage::decode_from(&mut cursor).unwrap();
                    println!("Received request: {}", req);

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
                    res.add_avp(avp!(263, None, UTF8StringAvp::new("ses;12345888"), true));
                    res.add_avp(avp!(416, None, EnumeratedAvp::new(1), true));
                    res.add_avp(avp!(415, None, Unsigned32Avp::new(1000), true));
                    res.add_avp(avp!(268, None, Unsigned32Avp::new(2001), true));

                    // Encode Response
                    let mut response = Vec::new();
                    res.encode_to(&mut response).unwrap();

                    // Send Response
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

    #[ignore]
    #[tokio::test]
    async fn test_server() {
        let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();
        server.handle().await.unwrap();
    }
}
