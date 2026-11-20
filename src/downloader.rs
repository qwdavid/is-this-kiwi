/// `url` のファイルをダウンロードして `path` に保存する。
/// 親ディレクトリがなければ作成する。
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
