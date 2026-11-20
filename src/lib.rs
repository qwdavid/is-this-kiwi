mod commands;
mod downloader;
mod handler;
mod python_bridge;

use std::error::Error;

use dotenv::dotenv;
use serenity::prelude::*;

pub async fn run_app() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN")?;

    // スラッシュコマンドのみを使うので Intent は空でよい
    let intents = GatewayIntents::empty();
    let mut client = Client::builder(token, intents)
        .event_handler(handler::Handler)
        .await?;

    client.start().await?;
    Ok(())
}
