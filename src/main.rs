use std::sync::Arc;

pub use error::Result;

mod app;
mod audio;
mod config;
mod error;
mod net;
mod peer;
mod ui;

#[tokio::main]
async fn main() -> Result<()> {
    let config = config::Config::load()?;
    let (app, sub_apps) = app::App::new(config).await?;
    Arc::new(app).run(sub_apps).await;
    Ok(())
}
