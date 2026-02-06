#[tokio::main]
async fn main() -> Result<(),Box<dyn std::error::Error>>{
    discord_bot::run_app().await?;
    Ok(())
}
