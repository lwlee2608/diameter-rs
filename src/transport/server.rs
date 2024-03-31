//! Diameter Protocol Server
use crate::diameter::DiameterMessage;
use crate::error::Result;
use crate::transport::Codec;
use std::future::Future;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

pub struct DiameterServerConfig {
    pub native_tls: Option<native_tls::Identity>,
}
/// A Diameter protocol server for handling Diameter requests and responses.
///
/// This server listens for incoming Diameter messages, processes them, and sends back responses.
pub struct DiameterServer {
    listener: TcpListener,
    config: DiameterServerConfig,
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
    pub async fn new(addr: &str, config: DiameterServerConfig) -> Result<DiameterServer> {
        let listener = TcpListener::bind(addr).await?;
        Ok(DiameterServer { listener, config })
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
    pub async fn listen<F, Fut>(&mut self, handler: F) -> Result<()>
    where
        F: Fn(DiameterMessage) -> Fut + Clone + Send + 'static,
        Fut: Future<Output = Result<DiameterMessage>> + Send + 'static,
    {
        loop {
            match self.config.native_tls {
                Some(ref identity) => {
                    let acceptor = native_tls::TlsAcceptor::new(identity.clone()).unwrap();
                    let acceptor = tokio_native_tls::TlsAcceptor::from(acceptor);

                    let (stream, peer_addr) = self.listener.accept().await?;
                    let stream = acceptor.accept(stream).await.unwrap();

                    Self::handle_peer(peer_addr, stream, handler.clone()).await?;
                }
                None => {
                    let (stream, peer_addr) = self.listener.accept().await?;
                    Self::handle_peer(peer_addr, stream, handler.clone()).await?;
                }
            };
        }
    }

    async fn handle_peer<F, Fut, S>(peer_addr: SocketAddr, stream: S, handler: F) -> Result<()>
    where
        F: Fn(DiameterMessage) -> Fut + Clone + Send + 'static,
        Fut: Future<Output = Result<DiameterMessage>> + Send + 'static,
        S: AsyncReadExt + AsyncWriteExt + Unpin + Send + 'static,
    {
        let handler = handler.clone();
        tokio::spawn(async move {
            log::info!("[{}] Connection established", peer_addr);
            match Self::process_incoming_message(stream, handler).await {
                Ok(_) => {
                    log::info!("[{}] Connection closed", peer_addr);
                }
                Err(e) => {
                    log::error!("Fatal error occurred: {:?}", e);
                }
            }
        });
        todo!()
    }

    async fn process_incoming_message<F, Fut, S>(mut stream: S, handler: F) -> Result<()>
    where
        F: Fn(DiameterMessage) -> Fut,
        Fut: Future<Output = Result<DiameterMessage>>,
        S: AsyncReadExt + AsyncWriteExt + Unpin,
    {
        // let (mut reader, mut writer) = stream.split();
        loop {
            // Read and decode the request
            let req = match Codec::decode(&mut stream).await {
                Ok(req) => req,
                Err(e) => match e {
                    crate::error::Error::IoError(ref e)
                        if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                    {
                        return Ok(());
                    }
                    _ => {
                        return Err(e);
                    }
                },
            };

            // Process the request using the handler
            let res = handler(req).await?;

            // Encode and send the response
            Codec::encode(&mut stream, &res).await?;
        }
    }
}
