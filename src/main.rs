#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    is_this_kiwi::run_app().await?;
    Ok(())
}
