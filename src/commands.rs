use serenity::{
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    model::prelude::*,
    prelude::*,
};

use crate::{downloader, python_bridge};

/// Discord に登録するスラッシュコマンド名
pub const COMMAND_NAME: &str = "is_this_kiwi";

/// 添付画像を解析前に保存しておく一時ファイルのパスを、リクエストごとに作る。
///
/// 以前は全リクエスト共通の固定パス（`tmp/input.png`）だったため、同時に
/// コマンドが使われると後から来た画像が先の画像を上書きし、別人の画像を
/// 判定してしまうことがあった。インタラクション ID と添付ファイル ID は
/// Discord 側で一意に発番されるので、その組み合わせをファイル名に使う。
fn input_image_path(interaction_id: u64, attachment_id: u64) -> String {
    format!("tmp/input-{interaction_id}-{attachment_id}.png")
}

pub fn create_command() -> CreateCommand {
    CreateCommand::new(COMMAND_NAME)
        .description("鳥かフルーツかを判断するコマンド")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Attachment, "image", "画像ファイル")
                .required(true),
        )
}

pub async fn handle_is_this_kiwi(ctx: &Context, command: &CommandInteraction) {
    let Some(option) = command.data.options.first() else {
        return;
    };

    let CommandDataOptionValue::Attachment(attachment_id) = &option.value else {
        return;
    };

    let Some(attachment) = command.data.resolved.attachments.get(attachment_id) else {
        return;
    };

    // Discord にタイムアウトとみなされないよう、先に応答を返しておく
    respond(ctx, command, "画像を受け取りました。解析を開始します...").await;

    // このリクエスト専用の一時ファイルに保存する（同時リクエストと混ざらない）
    let input_path = input_image_path(command.id.get(), attachment_id.get());

    if let Err(e) = downloader::download_image(&attachment.url, &input_path).await {
        eprintln!("Error downloading image: {:?}", e);
        // ダウンロード途中のファイルが残っていれば片付ける
        let _ = tokio::fs::remove_file(&input_path).await;
        if let Err(err) = command
            .channel_id
            .say(
                &ctx.http,
                format!("画像のダウンロードに失敗しました: {:?}", e),
            )
            .await
        {
            eprintln!("Error sending failure message: {:?}", err);
        }
        return;
    }

    let python_result = python_bridge::call_python(&input_path);

    // 解析が終わったら、成功・失敗どちらでも一時ファイルを片付ける
    if let Err(e) = tokio::fs::remove_file(&input_path).await {
        if e.kind() != std::io::ErrorKind::NotFound {
            eprintln!("Error removing temporary image {}: {:?}", input_path, e);
        }
    }

    let result = match python_result {
        Ok(r) => r,
        Err(e) => {
            if let Err(err) = command
                .channel_id
                .say(&ctx.http, format!("Python実行失敗: {:?}", e))
                .await
            {
                eprintln!("Error sending failure message: {:?}", err);
            }
            return;
        }
    };

    // 解析には3秒以上かかる場合があるため、結果はチャンネルへの新規メッセージで送る
    if let Err(e) = command
        .channel_id
        .say(
            &ctx.http,
            format!("受け取った画像: {}\n分析結果: {}", attachment.url, result),
        )
        .await
    {
        eprintln!("Error sending result message: {:?}", e);
    }
}

async fn respond(ctx: &Context, command: &CommandInteraction, content: &str) {
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new().content(content),
    );

    if let Err(e) = command.create_response(&ctx.http, response).await {
        eprintln!("Error sending message: {:?}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::input_image_path;

    #[test]
    fn input_image_path_is_unique_per_request() {
        // 別リクエスト（別インタラクション / 別添付）ではパスが重ならない
        assert_ne!(input_image_path(1, 100), input_image_path(2, 100));
        assert_ne!(input_image_path(1, 100), input_image_path(1, 200));
        // 同じリクエストでは決定的に同じパスになる
        assert_eq!(input_image_path(1, 100), input_image_path(1, 100));
    }

    #[test]
    fn input_image_path_stays_under_tmp() {
        assert!(input_image_path(42, 7).starts_with("tmp/input-"));
    }
}
