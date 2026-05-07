use crate::{Result, app::App, config::Config};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::{net::UdpSocket, sync::mpsc};

pub struct NetApp {
    socket: UdpSocket,
    local_name: String,
    peers: Vec<SocketAddr>,
    record_rx: mpsc::Receiver<Vec<u8>>,
}
impl NetApp {
    const PREFIX: &'static [u8] = b"VOICELAND\n";

    pub async fn new(
        config: &Config,
        tx: mpsc::Sender<String>,
        record_rx: mpsc::Receiver<Vec<u8>>,
    ) -> Result<Self> {
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, config.port)).await?;
        tx.send(format!(
            "{} listen on port {}",
            config.local_name, config.port
        ))
        .await?;
        Ok(Self {
            socket,
            local_name: config.local_name.clone(),
            peers: config
                .peers
                .iter()
                .map(|addr| (*addr, config.port).into())
                .collect(),
            record_rx,
        })
    }

    pub async fn run(&mut self, app: &App) {
        let mut buf = [0; 1536];
        loop {
            tokio::select! {
                Some(voice_data) = self.record_rx.recv() => {
                    self.send_voice(&voice_data).await.unwrap();
                },
                result = self.socket.recv_from(&mut buf) => {
                    if let Ok((len, addr)) = result {
                        let data = &buf[..len];
                        if data.starts_with(Self::PREFIX) {
                            let name_len = data[Self::PREFIX.len()] as usize;
                            // let name = String::from_utf8_lossy(
                            //     &data[Self::PREFIX.len() + 1..Self::PREFIX.len() + 1 + name_len],
                            // );
                            let voice_data = data[Self::PREFIX.len() + 1 + name_len..].to_vec();

                            // if pear exist
                            let idx;
                            {
                                let peers = app.peers.read().await;
                                idx = peers.binary_search_by_key(&addr, |p| p.addr);
                                if let Ok(idx) = idx {
                                    peers[idx].receive_voice(voice_data).await;
                                    continue;
                                }
                            }
                            {
                                let mut peers = app.peers.write().await;
                                if let Err(idx) = idx {
                                    peers.insert(
                                        idx,
                                        crate::peer::Peer::new(
                                            String::from_utf8_lossy(
                                                &data[Self::PREFIX.len() + 1
                                                    ..Self::PREFIX.len() + 1 + name_len],
                                            )
                                            .to_string(),
                                            addr,
                                        ),
                                    );
                                    self.add_peer(addr).await;
                                    peers[idx].receive_voice(voice_data).await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn add_peer(&mut self, addr: SocketAddr) {
        self.peers.push(addr);
    }

    pub async fn send_voice(&mut self, voice_data: &[u8]) -> Result<()> {
        let mut data =
            Vec::with_capacity(Self::PREFIX.len() + 1 + self.local_name.len() + voice_data.len());
        data.extend_from_slice(Self::PREFIX);
        data.push(self.local_name.len() as u8);
        data.extend_from_slice(self.local_name.as_bytes());
        data.extend_from_slice(voice_data);

        for addr in &self.peers {
            self.socket.send_to(&data, addr).await?;
        }
        Ok(())
    }
}
