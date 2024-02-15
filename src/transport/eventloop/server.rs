//! Diameter Protocol Server
use crate::diameter::DiameterMessage;
use crate::error::Result;
use crate::transport::Codec;
use log::error;
use tokio::net::TcpListener;
use tokio::task;

/// A Diameter protocol server for handling Diameter requests and responses.
///
/// This server listens for incoming Diameter messages, processes them, and sends back responses.
pub struct DiameterServer {
    listener: TcpListener,
}

impl DiameterServer {
    /// Creates a new `DiameterServer` and starts listening on the specified address.
    ///
    /// This method binds to the given address and starts listening for incoming connections.
    ///
    /// Args:
    ///     addr: The address on which the server should listen.
    ///
    /// Returns:
    ///     A `Result` containing the new `DiameterServer` instance or an `Error` if the binding fails.
    pub async fn new(addr: &str) -> Result<DiameterServer> {
        let listener = TcpListener::bind(addr).await?;
        Ok(DiameterServer { listener })
    }

    /// Listens for incoming connections and processes Diameter messages.
    ///
    /// This method continuously accepts new connections, reads incoming Diameter messages,
    /// uses the provided handler to process them, and sends back the responses.
    ///
    /// The server will listen indefinitely, handling each incoming connection in a loop.
    /// Each connection is handled in its own asynchronous task.
    ///
    /// Args:
    ///     handler: A function or closure that takes a `DiameterMessage` and returns a `Result`
    ///              with either the response `DiameterMessage` or an `Error`. This handler
    ///              is responsible for processing the incoming messages and determining the
    ///              appropriate responses.
    ///
    /// Returns:
    ///     A `Result` indicating the success or failure of the operation. Errors could occur
    ///     during the acceptance of new connections or during the message handling process.
    pub async fn listen<F>(&mut self, handler: F) -> Result<()>
    where
        F: Fn(DiameterMessage) -> Result<DiameterMessage> + Clone + Send + 'static,
    {
        loop {
            let (mut stream, _) = self.listener.accept().await?;

            let peer_addr = match stream.peer_addr() {
                Ok(addr) => addr.to_string(),
                Err(_) => "Unknown".to_string(),
            };

            let handler = handler.clone();
            task::spawn_local(async move {
                let (mut reader, mut writer) = stream.split();
                loop {
                    // Read and decode the request
                    let req = match Codec::decode(&mut reader).await {
                        Ok(req) => req,
                        Err(e) => {
                            error!(
                                "[{}] Failed to read and decode message; err = {:?}",
                                peer_addr, e
                            );
                            return;
                        }
                    };

                    // Process the request using the handler
                    let res = match handler(req) {
                        Ok(res) => res,
                        Err(e) => {
                            error!("[{}] Request handler error: {:?}", peer_addr, e);
                            return;
                        }
                    };

                    // Encode and send the response
                    if let Err(e) = Codec::encode(&mut writer, &res).await {
                        error!(
                            "[{}] Failed to encode and send response; err = {:?}",
                            peer_addr, e
                        );
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
    use crate::avp::enumerated::Enumerated;
    use crate::avp::flags::M;
    use crate::avp::identity::Identity;
    use crate::avp::unsigned32::Unsigned32;
    use crate::avp::utf8string::UTF8String;
    use crate::avp::Avp;
    use crate::diameter::{ApplicationId, CommandCode, DiameterMessage};

    #[ignore]
    #[tokio::test]
    async fn test_server() {
        let mut server = DiameterServer::new("0.0.0.0:3868").await.unwrap();
        server
            .listen(|_req| -> Result<DiameterMessage> {
                // Return Dummy Value
                let mut res = DiameterMessage::new(
                    CommandCode::CreditControl,
                    ApplicationId::CreditControl,
                    0,
                    1123158610,
                    3102381851,
                );
                res.add_avp(avp!(264, None, M, Identity::new("host.example.com")));
                res.add_avp(avp!(296, None, M, Identity::new("realm.example.com")));
                res.add_avp(avp!(263, None, M, UTF8String::new("ses;12345889")));
                res.add_avp(avp!(416, None, M, Enumerated::new(1)));
                res.add_avp(avp!(415, None, M, Unsigned32::new(1000)));
                res.add_avp(avp!(268, None, M, Unsigned32::new(2001)));
                Ok(res)
            })
            .await
            .unwrap();
    }
}
