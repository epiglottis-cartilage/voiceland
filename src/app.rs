use std::sync::{Arc, atomic::AtomicU16};

use crate::{Result, audio::AudioApp, config::Config, net::NetApp, peer::Peer, ui::UiApp};
use tokio::sync::RwLock;

pub struct App {
    // pub running: AtomicBool,
    pub peers: RwLock<Vec<Peer>>,
    pub name: String,
    /// 100 -> 100%
    pub volume: Arc<AtomicU16>,
}

impl App {
    pub async fn new(config: Config) -> Result<(Self, (NetApp, UiApp, AudioApp))> {
        let (log_tx, log_rx) = tokio::sync::mpsc::channel(100);
        let (record_tx, record_rx) = tokio::sync::mpsc::channel(100);
        let volume = Arc::new(AtomicU16::new(config.microphone_volume.unwrap_or(150)));
        let ui_app = UiApp::new(&config, log_rx);
        let net_app = NetApp::new(&config, log_tx.clone(), record_rx).await?;
        let audio_app = AudioApp::new(&config, log_tx.clone(), record_tx, volume.clone()).await?;
        Ok((
            Self {
                // running: AtomicBool::new(true),
                peers: RwLock::new(Vec::new()),
                name: config.name,
                volume,
            },
            (net_app, ui_app, audio_app),
        ))
    }

    pub async fn run(
        self: Arc<Self>,
        (mut net_app, mut ui_app, mut audio_app): (NetApp, UiApp, AudioApp),
    ) -> bool {
        let mut join_set = tokio::task::JoinSet::new();

        let app = self.clone();
        join_set.spawn(async move { ui_app.run(&*app).await });

        let app = self.clone();
        join_set.spawn(async move { net_app.run(&*app).await });

        let app = self.clone();
        join_set.spawn(async move { audio_app.run(&*app).await });

        let restart;
        match join_set.join_next().await {
            Some(Ok(false)) => {
                restart = false;
            }
            _ => {
                restart = true;
            }
        }
        join_set.shutdown().await;
        restart
    }
}
