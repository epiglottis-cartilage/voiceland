use crate::{Result, app::App, config::Config};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::{net::UdpSocket, sync::mpsc};

pub struct NetApp {
    socket: UdpSocket,
    local_name: String,
    peers: Vec<SocketAddr>,
    record_rx: mpsc::Receiver<Vec<u8>>,
    log_tx: mpsc::Sender<String>,
    buffer_len: u8,
    seq: u64,
}
impl NetApp {
    const PREFIX: &'static [u8] = b"VLAND\n";

    pub async fn new(
        config: &Config,
        tx: mpsc::Sender<String>,
        record_rx: mpsc::Receiver<Vec<u8>>,
    ) -> Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, config.port)).await?;
        tx.send(format!("{} listen on port {}", config.name, config.port))
            .await?;
        Ok(Self {
            socket,
            local_name: config.name.clone(),
            peers: config
                .peers
                .iter()
                .map(|addr| (*addr, config.port).into())
                .collect(),
            record_rx,
            log_tx: tx,
            buffer_len: config.buffer_len as u8,
            seq: 0,
        })
    }

    pub async fn run(&mut self, app: &App) -> bool {
        let mut buf = [0; 1500];
        loop {
            // if !app.running.load(std::sync::atomic::Ordering::Relaxed) {
            //     break true;
            // }

            tokio::select! {
                Some(voice_data) = self.record_rx.recv() => {
                    self.send_voice(&voice_data).await.unwrap();
                    self.seq += 1;
                },
                result = self.socket.recv_from(&mut buf) => {
                    if let Ok((len, addr)) = result {
                        let data = &buf[..len];
                        if data.starts_with(Self::PREFIX) && len > Self::PREFIX.len() + 8 + 1 {

                            let seq = u64::from_le_bytes(*data[Self::PREFIX.len()..].first_chunk().unwrap());
                            let name_len = data[Self::PREFIX.len() + 8] as usize;

                            let voice_data = match data.get(Self::PREFIX.len() + 8 + 1 + name_len..){
                                Some(x) => x.to_vec(),
                                None => continue,
                            };

                            // if pear exist
                            let idx;
                            {
                                let peers = app.peers.read().await;
                                idx = peers.binary_search_by_key(&addr, |p| p.addr);
                                if let Ok(idx) = idx {
                                    peers[idx].receive_voice(seq, voice_data, self.buffer_len);
                                    continue;
                                }
                            }
                            {
                                let mut peers = app.peers.write().await;
                                if let Err(idx) = idx {
                                    let name = String::from_utf8_lossy(
                                                &data[Self::PREFIX.len() + 8 + 1
                                                    ..Self::PREFIX.len() + 8 + 1 + name_len],
                                            ).to_string();
                                    if self.log_tx.send(format!("New peer: {} ({})", name, addr)).await.is_err(){
                                        break false;
                                    }
                                    peers.insert(
                                        idx,
                                        crate::peer::Peer::new(
                                            name,
                                            addr,
                                        ),
                                    );
                                    self.add_peer(addr).await;
                                    peers[idx].receive_voice(seq, voice_data, self.buffer_len);
                                }else{unreachable!()}
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn add_peer(&mut self, addr: SocketAddr) {
        if !self.peers.contains(&addr) {
            self.peers.push(addr);
        }
    }

    pub async fn send_voice(&mut self, voice_data: &[u8]) -> Result<()> {
        let mut data = Vec::with_capacity(
            Self::PREFIX.len() + 8 + 1 + self.local_name.len() + voice_data.len(),
        );
        data.extend(Self::PREFIX);
        data.extend(self.seq.to_le_bytes());
        data.push(self.local_name.len() as u8);
        data.extend(self.local_name.as_bytes());
        data.extend(voice_data);

        for addr in &self.peers {
            self.socket.send_to(&data, addr).await?;
        }
        Ok(())
    }
}
