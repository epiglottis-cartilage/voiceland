use std::{net::SocketAddr, sync::atomic::AtomicU8};
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Peer {
    pub name: String,
    pub addr: SocketAddr,
    pub volume: AtomicU8, // 0-255%,
    voice: Mutex<Option<Vec<u8>>>,
}

impl Peer {
    pub fn new(name: String, addr: SocketAddr) -> Self {
        Self {
            name,
            addr,
            volume: AtomicU8::new(100),
            voice: Mutex::new(None),
        }
    }
    pub async fn receive_voice(&self, data: Vec<u8>) {
        self.voice.lock().await.replace(data);
    }
}
