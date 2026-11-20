use serenity::{
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    model::prelude::*,
    prelude::*,
};

use crate::{downloader, python_bridge};

/// Name of the slash command registered with Discord.
pub const COMMAND_NAME: &str = "is_this_kiwi";

/// Path of the temporary file the attached image is saved to before analysis.
const INPUT_IMAGE_PATH: &str = "tmp/input.png";

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

    // ACK quickly so Discord doesn't think the interaction timed out.
    respond(ctx, command, "画像を受け取りました。解析を開始します...").await;

    if let Err(e) = downloader::download_image(&attachment.url, INPUT_IMAGE_PATH).await {
        eprintln!("Error downloading image: {:?}", e);
        // notify channel about failure
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

    let result = match python_bridge::call_python(INPUT_IMAGE_PATH) {
        Ok(r) => r,
        Err(e) => {
            // notify channel about python failure
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

    // send final result to the channel (can take longer than 3 seconds)
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
