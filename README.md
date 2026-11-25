# is-this-kiwi

Discordスラッシュコマンド `/is_this_kiwi` で、画像がキーウィ（鳥）かキウイ（果物）かを判定するボット。

## 仕組み

1. Discordで画像が添付されると、Rust（serenity）が画像を `tmp/` にリクエストごとの一意なファイル名（`input-<インタラクションID>-<添付ファイルID>.png`）でダウンロードする。同時リクエストで画像が混ざらないようにするためで、判定後にファイルは削除される
2. Python（TensorFlow / Keras）の判定モデル（`model/`）を呼び出して分類する
3. 結果（キーウィ / キウイ）をチャンネルに投稿する

## セットアップ

必要なもの: Rust、Python 3 + TensorFlow

```sh
pip install -r python/requirements.txt
```

`.env` に Discord のボットトークンを書く:

```
DISCORD_TOKEN=<redacted>
```

任意の環境変数:

- `GUILD_ID` — コマンドを登録するサーバーを1つに絞る（未設定なら参加中の全サーバー）
- `PY_ANALYZE_SCRIPT` — 解析スクリプトのパス（デフォルト: `python/analyze.py`）

## 実行

```sh
cargo run
```

## 構成

- `src/` — Discordボット本体（Rust / serenity）
  - `main.rs` — エントリポイント
  - `lib.rs` — アプリ起動
  - `handler.rs` — Discordイベントの処理
  - `commands.rs` — `/is_this_kiwi` コマンドの定義と処理
  - `downloader.rs` — 添付画像のダウンロード
  - `python_bridge.rs` — Python解析スクリプトの呼び出し
- `python/analyze.py` — TensorFlowによる画像分類
- `model/` — 学習済みモデル（`model_predict.json` / `model_predict.weights.h5`）
