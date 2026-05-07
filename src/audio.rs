use crate::{Result, app::App};
use tokio::sync::mpsc;

pub struct AudioApp {
    // audio handling fields
    record_tx: mpsc::Sender<Vec<u8>>,
}
impl AudioApp {
    pub async fn new(tx: mpsc::Sender<String>, record_tx: mpsc::Sender<Vec<u8>>) -> Result<Self> {
        // initialize audio handling
        tx.send("Audio initialized".to_string()).await?;
        Ok(Self { record_tx })
    }

    pub async fn run(&mut self, app: &App) {
        // main audio processing loop
        loop {
            app;
        }
    }
}
