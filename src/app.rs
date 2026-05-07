use std::sync::{Arc, atomic::AtomicBool};

use crate::{Result, audio::AudioApp, config::Config, net::NetApp, peer::Peer, ui::UiApp};
use tokio::sync::RwLock;

pub struct App {
    pub running: AtomicBool,
    pub peers: RwLock<Vec<Peer>>,
}

impl App {
    pub async fn new(config: Config) -> Result<(Self, (NetApp, UiApp, AudioApp))> {
        let (log_tx, log_rx) = tokio::sync::mpsc::channel(100);
        let (record_tx, record_rx) = tokio::sync::mpsc::channel(100);

        let ui_app = UiApp::new(log_rx);
        let net_app = NetApp::new(&config, log_tx.clone(), record_rx).await?;
        let audio_app = AudioApp::new(log_tx.clone(), record_tx).await?;
        Ok((
            Self {
                running: AtomicBool::new(true),
                peers: RwLock::new(Vec::new()),
            },
            (net_app, ui_app, audio_app),
        ))
    }

    pub async fn run(
        self: Arc<Self>,
        (mut net_app, mut ui_app, mut audio_app): (NetApp, UiApp, AudioApp),
    ) {
        let app = self.clone();
        let net_handle = tokio::spawn(async move {
            net_app.run(&*app).await;
        });

        let app = self.clone();
        let ui_handle = tokio::spawn(async move {
            ui_app.run(&*app).await;
        });

        let app = self.clone();
        let audio_handle = tokio::spawn(async move {
            audio_app.run(&*app).await;
        });

        let _ = tokio::join!(net_handle, ui_handle, audio_handle);
    }
}
