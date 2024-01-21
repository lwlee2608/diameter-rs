use crate::diameter::DiameterMessage;
use crate::error::Error;
use log::error;
use std::io::Cursor;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;

/// A Diameter protocol server for handling Diameter requests and responses.
///
/// This server listens for incoming Diameter messages, processes them, and sends back responses.
///
/// Fields:
///     listener: The TCP listener that accepts incoming connections.
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
    pub async fn new(addr: &str) -> Result<DiameterServer, Error> {
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
    pub async fn listen<F>(&mut self, handler: F) -> Result<(), Error>
    where
        F: Fn(DiameterMessage) -> Result<DiameterMessage, Error> + Clone + Send + 'static,
    {
        loop {
            let (stream, _) = self.listener.accept().await?;

            let peer_addr = match stream.peer_addr() {
                Ok(addr) => addr.to_string(),
                Err(_) => "Unknown".to_string(),
            };

            let (mut reader, mut writer) = stream.into_split();

            let handler = handler.clone();
            tokio::spawn(async move {
                loop {
                    // Read and decode the request
                    let req = match Self::read_and_decode_message(&mut reader).await {
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
                    if let Err(e) = Self::encode_and_send_message(&mut writer, res).await {
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

    async fn read_and_decode_message(reader: &mut OwnedReadHalf) -> Result<DiameterMessage, Error> {
        // Read first 4 bytes to determine the length
        let mut b = [0; 4];
        reader.read_exact(&mut b).await?;
        let length = u32::from_be_bytes([0, b[1], b[2], b[3]]);

        // Limit to 1MB
        if length > 1024 * 1024 {
            return Err(Error::ServerError("Message too large".into()));
        }

        // Read the rest of the message
        let mut buf = vec![0; length as usize - 4];
        reader.read_exact(&mut buf).await?;

        let mut request = Vec::with_capacity(length as usize);
        request.extend_from_slice(&b);
        request.append(&mut buf);

        // Decode the message
        let mut cursor = Cursor::new(request);
        DiameterMessage::decode_from(&mut cursor)
    }

    async fn encode_and_send_message(
        writer: &mut OwnedWriteHalf,
        msg: DiameterMessage,
    ) -> Result<(), Error> {
        // Encode and send the response
        let mut response = Vec::new();
        msg.encode_to(&mut response)?;

        // Send the response
        writer.write_all(&response).await?;

        Ok(())
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
            .listen(|_req| -> Result<DiameterMessage, Error> {
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
