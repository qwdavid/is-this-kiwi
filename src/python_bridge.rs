use std::process::Command;
use std::error::Error;
use regex::Regex;
use encoding_rs::SHIFT_JIS;
use std::env;

fn decode_bytes(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    let (cow, _enc_used, _had_errors) = SHIFT_JIS.decode(bytes);
    cow.into_owned()
}

/// Python の解析スクリプトを呼び出す。
/// スクリプトのパスは環境変数 `PY_ANALYZE_SCRIPT` で変更でき、デフォルトは `python/analyze.py`。
pub fn call_python(image_path: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let script = env::var("PY_ANALYZE_SCRIPT").unwrap_or_else(|_| "python/analyze.py".to_string());

    let output = Command::new("python")
        .arg(&script)
        .arg(image_path)
        .output()?;

    if !output.status.success() {
        let err_text = decode_bytes(&output.stderr);
        return Err(format!("Python error: {}", err_text).into());
    }

    let raw = decode_bytes(&output.stdout);

    // ANSI エスケープシーケンスを取り除く
    let ansi_re = Regex::new(r"\x1B\[[0-9;?]*[ -/]*[@-~]").unwrap();
    let mut cleaned = ansi_re.replace_all(&raw, "").to_string();

    // 表示可能な文字と改行・タブだけを残す
    cleaned.retain(|c| {
        let code = c as u32;
        code == 0x0A || code == 0x09 || (code >= 0x20 && code != 0x7F)
    });

    Ok(cleaned.trim().to_string())
}
