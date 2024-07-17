//! Diameter Protocol Client
use crate::diameter::DiameterMessage;
use crate::dictionary::Dictionary;
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
    ///    * handler: The `ClientHandler` for reading messages from the server.
    ///    * dictionary: The `Dictionary` for decoding messages.
    ///
    /// Example:
    ///    ```no_run
    ///    use diameter::transport::client::{ClientHandler, DiameterClient, DiameterClientConfig};
    ///    use diameter::dictionary;
    ///    use std::sync::Arc;
    ///
    ///    #[tokio::main]
    ///    async fn main() {
    ///        let dict = dictionary::Dictionary::new(&[&dictionary::DEFAULT_DICT_XML]);
    ///        let dict = Arc::new(dict);
    ///        let config = DiameterClientConfig { use_tls: false, verify_cert: false };
    ///        let mut client = DiameterClient::new("localhost:3868", config);
    ///        let mut handler = client.connect().await.unwrap();
    ///        tokio::spawn(async move {
    ///            DiameterClient::handle(&mut handler, dict).await;
    ///        });
    ///    }
    ///    ```
    pub async fn handle(handler: &mut ClientHandler, dictionary: Arc<Dictionary>) {
        loop {
            match Codec::decode(&mut handler.reader, Arc::clone(&dictionary)).await {
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

    /// Sends a Diameter message and returns a future for receiving the response.
    ///
    /// Args:
    ///   req: The Diameter message to send.
    ///   Returns:
    ///   A `ResponseFuture` for receiving the response from the server.
    ///   The future will resolve to a `DiameterMessage` containing the response.
    ///
    pub async fn send_message(&mut self, req: DiameterMessage) -> Result<ResponseFuture> {
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

/// A future for receiving a Diameter message response.
///
#[derive(Debug)]
pub struct ResponseFuture {
    pub receiver: Receiver<DiameterMessage>,
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
