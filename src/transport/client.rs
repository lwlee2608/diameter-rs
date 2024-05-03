//! Diameter Protocol Client
use crate::diameter::DiameterMessage;
use crate::error::{Error, Result};
use crate::transport::Codec;
use std::collections::HashMap;
use std::future::Future;
use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::sync::Mutex;

/// Configuration for a Diameter protocol client.
///
pub struct DiameterClientConfig {
    pub use_tls: bool,
    pub verify_cert: bool,
    // pub native_tls: Option<native_tls::Identity>, // Future Implementation
}

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
    config: DiameterClientConfig,
    address: String,
    writer: Option<Arc<Mutex<dyn AsyncWrite + Send + Unpin>>>,
    msg_caches: Arc<Mutex<HashMap<u32, Sender<DiameterMessage>>>>,
    seq_num: u32,
}

impl DiameterClient {
    /// Creates a new `DiameterClient` instance with a specified server address.
    ///
    /// Initializes the internal structures but does not establish a connection.
    /// The connection to the server will be established when `connect` is called.
    ///
    /// Args:
    ///     addr: The address of the Diameter server to connect to.
    ///
    /// Returns:
    ///     A new instance of `DiameterClient`.
    pub fn new(addr: &str, config: DiameterClientConfig) -> DiameterClient {
        DiameterClient {
            config,
            address: addr.into(),
            writer: None,
            msg_caches: Arc::new(Mutex::new(HashMap::new())),
            seq_num: 0,
        }
    }

    /// Establishes a connection to the Diameter server.
    ///
    /// Returns:
    ///    A `Result` containing a `ClientHandler` or an error if the connection cannot be established.
    pub async fn connect(&mut self) -> Result<ClientHandler> {
        let stream = TcpStream::connect(self.address.clone()).await?;

        if self.config.use_tls {
            let tls_connector = tokio_native_tls::TlsConnector::from(
                native_tls::TlsConnector::builder()
                    .danger_accept_invalid_certs(!self.config.verify_cert)
                    .build()?,
            );
            let tls_stream = tls_connector.connect(&self.address.clone(), stream).await?;
            let (reader, writer) = tokio::io::split(tls_stream);

            // writer
            let writer = Arc::new(Mutex::new(writer));
            self.writer = Some(writer);

            // reader
            let msg_caches = Arc::clone(&self.msg_caches);
            Ok(ClientHandler {
                reader: Box::new(reader),
                msg_caches,
            })
        } else {
            let (reader, writer) = tokio::io::split(stream);

            // writer
            let writer = Arc::new(Mutex::new(writer));
            self.writer = Some(writer);

            // reader
            let msg_caches = Arc::clone(&self.msg_caches);
            Ok(ClientHandler {
                reader: Box::new(reader),
                msg_caches,
            })
        }
    }

    /// Handles incoming Diameter messages.
    ///
    /// This method reads incoming messages from the server and processes them.
    /// The method is intended to be run in a separate task.
    ///
    /// Args:
    ///    handler: The `ClientHandler` for reading messages from the server.
    ///
    /// Example:
    ///    ```no_run
    ///    use diameter::transport::client::{ClientHandler, DiameterClient, DiameterClientConfig};
    ///
    ///    #[tokio::main]
    ///    async fn main() {
    ///        let config = DiameterClientConfig { use_tls: false, verify_cert: false };
    ///        let mut client = DiameterClient::new("localhost:3868", config);
    ///        let mut handler = client.connect().await.unwrap();
    ///        tokio::spawn(async move {
    ///            DiameterClient::handle(&mut handler).await;
    ///        });
    ///    }
    ///    ```
    pub async fn handle(handler: &mut ClientHandler) {
        loop {
            match Codec::decode(&mut handler.reader).await {
                Ok(res) => {
                    if let Err(e) = Self::process_decoded_msg(handler.msg_caches.clone(), res).await
                    {
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

    /// Initiates a Diameter request.
    ///
    /// This method creates and caches a request, readying it for sending to the server.
    ///
    /// Args:
    ///     req: The Diameter message to send as a request.
    ///
    /// Returns:
    ///     A `Result` containing a `DiameterRequest` or an error if the client is not connected.
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

    /// Sends a Diameter message and waits for the response.
    ///
    /// This is a convenience method that combines sending a request and waiting for its response.
    ///
    /// Args:
    ///     req: The Diameter message to send.
    ///
    /// Returns:
    ///     A `Result` containing the response `DiameterMessage` or an error.
    pub async fn send_message(&mut self, req: DiameterMessage) -> Result<DiameterMessage> {
        let mut request = self.request(req).await?;
        let _ = request.send().await?;
        let response = request.response().await?;
        Ok(response)
    }

    pub async fn send_message_async(&mut self, req: DiameterMessage) -> Result<ResponseFuture> {
        if let Some(writer) = &self.writer {
            let (tx, rx) = oneshot::channel();
            let hop_by_hop = req.get_hop_by_hop_id();
            {
                let mut msg_caches = self.msg_caches.lock().await;
                msg_caches.insert(hop_by_hop, tx);
            }
            let mut writer = writer.lock().await;
            Codec::encode(&mut writer.deref_mut(), &req).await?;
            Ok(ResponseFuture { receiver: rx })
        } else {
            Err(Error::ClientError("Not connected".into()))
        }
    }

    // Returns the next sequence number.
    pub fn get_next_seq_num(&mut self) -> u32 {
        self.seq_num += 1;
        self.seq_num
    }
}

/// A Diameter protocol client handler for receiving Diameter messages.
///
pub struct ClientHandler {
    // reader: ReadHalf<TcpStream>,
    reader: Box<dyn AsyncRead + Send + Unpin>,
    msg_caches: Arc<Mutex<HashMap<u32, Sender<DiameterMessage>>>>,
}

/// Represents a single Diameter request and its associated response channel.
///
/// This structure is used to manage the lifecycle of a Diameter request,
/// including sending the request and receiving the response.
///
/// Fields:
///     request: The Diameter message representing the request.
///     receiver: A channel for receiving the response to the request.
///     writer: A thread-safe writer for sending the request to the server.
pub struct DiameterRequest {
    request: DiameterMessage,
    receiver: Arc<Mutex<Option<Receiver<DiameterMessage>>>>,
    writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
}

impl DiameterRequest {
    /// Creates a new `DiameterRequest`.
    ///
    /// Args:
    ///     request: The Diameter message to be sent as a request.
    ///     receiver: The channel receiver for receiving the response.
    ///     writer: A shared reference to the writer for sending the request.
    ///
    /// Returns:
    ///     A new instance of `DiameterRequest`.
    pub fn new(
        request: DiameterMessage,
        receiver: Receiver<DiameterMessage>,
        writer: Arc<Mutex<dyn AsyncWrite + Send + Unpin>>,
    ) -> Self {
        DiameterRequest {
            request,
            receiver: Arc::new(Mutex::new(Some(receiver))),
            writer,
        }
    }

    /// Returns a reference to the request message.
    ///
    /// This method allows access to the original request message.
    ///
    /// Returns:
    ///     A reference to the `DiameterMessage` representing the request.
    pub fn get_request(&self) -> &DiameterMessage {
        &self.request
    }

    /// Sends the request to the Diameter server.
    ///
    /// This method encodes and sends the request message to the server.
    ///
    /// Returns:
    ///     A `Result` indicating the success or failure of sending the request.
    pub async fn send(&mut self) -> Result<()> {
        let mut writer = self.writer.lock().await;
        Codec::encode(&mut writer.deref_mut(), &self.request).await
    }

    /// Waits for and returns the response to the request.
    ///
    /// This method waits for the response from the server to the request.
    ///
    /// Returns:
    ///     A `Result` containing the response `DiameterMessage` or an error if the response cannot be received.
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

#[derive(Debug)]
pub struct ResponseFuture {
    pub receiver: oneshot::Receiver<DiameterMessage>,
}

impl Future for ResponseFuture {
    type Output = Result<DiameterMessage>;

    fn poll(
        mut self: Pin<&mut Self>,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        match Pin::new(&mut self.receiver).poll(ctx) {
            std::task::Poll::Ready(result) => match result {
                Ok(response) => std::task::Poll::Ready(Ok(response)),
                Err(_) => std::task::Poll::Ready(Err(Error::ClientError(
                    "Response channel closed".into(),
                ))),
            },
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
