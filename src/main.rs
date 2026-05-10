use std::sync::Arc;

pub use error::Result;

mod app;
mod audio;
mod codec;
mod config;
mod error;
mod net;
mod peer;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    loop {
        let config = config::Config::load("voiceland.toml")?;
        let (app, sub_apps) = app::App::new(config).await?;
        let restart = Arc::new(app).run(sub_apps).await;
        if !restart {
            break;
        }
    }
    Ok(())
}
