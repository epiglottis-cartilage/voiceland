use std::sync::Mutex;
use std::sync::atomic::AtomicU16;
use std::{collections::VecDeque, net::SocketAddr};

#[derive(Debug)]
pub struct Peer {
    pub name: String,
    pub addr: SocketAddr,
    pub volume: AtomicU16,
    voice: Mutex<VecDeque<Vec<u8>>>,
}

impl Peer {
    pub fn new(name: String, addr: SocketAddr) -> Self {
        Self {
            name,
            addr,
            volume: AtomicU16::new(100),
            voice: Mutex::new(VecDeque::new()),
        }
    }

    pub fn receive_voice(&self, data: Vec<u8>, ttl: u8) {
        let mut queue = self.voice.lock().unwrap();
        queue.push_back(data);
        while queue.len() > ttl as usize {
            queue.pop_front();
        }
    }

    pub fn try_pop_voice(&self) -> Option<Vec<u8>> {
        self.voice.lock().unwrap().pop_front()
    }
}
