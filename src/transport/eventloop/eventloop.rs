#![allow(dead_code)]
// TODO REMOVE ME

use std::thread;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub enum Event {
    Connect,
    Disconnect,
    Data(Vec<u8>),
}

pub struct EventLoop {
    main_loop: tokio::task::JoinHandle<()>,
}

impl EventLoop {
    pub fn new() -> (Self, WriteHandler) {
        let (send, recv) = channel(64);

        let handler = WriteHandler { send };

        let main_loop = tokio::spawn(async move {
            Self::main_loop(recv).await;
        });

        (EventLoop { main_loop }, handler)
    }

    pub fn abort(&self) {
        self.main_loop.abort();
    }

    async fn main_loop(mut recv: Receiver<Event>) {
        while let Some(event) = recv.recv().await {
            match event {
                Event::Connect => {
                    // Connect to the server
                    println!("[{:?}] Connect", thread::current().id());
                }
                Event::Disconnect => {
                    // Disconnect from the server
                    println!("[{:?}] Disconnect", thread::current().id());
                }
                Event::Data(_data) => {
                    // Send the data to the server
                    println!("[{:?}] Data", thread::current().id());
                }
            }
        }
    }
}

pub struct WriteHandler {
    send: Sender<Event>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_event_loop() {
        let (event_loop, handler) = EventLoop::new();

        handler.send.send(Event::Connect).await.unwrap();
        handler.send.send(Event::Data(vec![1, 2, 3])).await.unwrap();
        handler.send.send(Event::Disconnect).await.unwrap();

        sleep(Duration::from_millis(10)).await;

        event_loop.abort();
    }
}
