/// Download the file at `url` and save it to `path`,
/// creating the parent directory if it does not exist yet.
pub async fn download_image(
    url: &str,
    path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }

    let bytes = reqwest::get(url).await?.bytes().await?;
    tokio::fs::write(path, &bytes).await?;
    Ok(())
}
