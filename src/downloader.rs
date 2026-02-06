use std::fs::File;
use std::io::Write;

pub async fn download_image(url: &str, path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bytes = reqwest::get(url).await?.bytes().await?;
    let mut file = File::create(path)?;
    file.write_all(&bytes)?;
    Ok(())
}
