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

    let intents = GatewayIntents::empty(); // スラッシュコマンドだけなら基本空でOK
    let mut client = Client::builder(token, intents)
        .event_handler(handler::Handler)
        .await?;

    client.start().await?;
    Ok(())
}
