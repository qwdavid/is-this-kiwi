use std::error::Error;
use dotenv::dotenv;
use serenity::{
    async_trait,
    builder::{
        CreateCommand, CreateCommandOption,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    model::prelude::*,
    prelude::*,
};

pub mod downloader;
pub use downloader::download_image;
pub mod python_bridge;
pub use python_bridge::call_python;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        match command.data.name.as_str() {
            "is_this_kiwi" | "sendimage" => self.handle_sendimage(&ctx, &command).await,
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        // Register the command. If `GUILD_ID` env var is set to a numeric guild id,
        // register only there. Otherwise register the command in every guild the
        // bot is cached in (so it's available on all servers the bot has joined).
        let command_name = "is_this_kiwi";

        let guild_ids: Vec<GuildId> = match std::env::var("GUILD_ID") {
            Ok(s) => match s.parse::<u64>() {
                Ok(n) => vec![GuildId::new(n)],
                Err(_) => ctx.cache.guilds().iter().cloned().collect(),
            },
            Err(_) => ctx.cache.guilds().iter().cloned().collect(),
        };

        for gid in guild_ids {
            let res = gid
                .create_command(&ctx.http, CreateCommand::new(command_name)
                    .description("鳥かフルーツかを判断するコマンド")
                    .add_option(
                        CreateCommandOption::new(
                            CommandOptionType::Attachment,
                            "image",
                            "画像ファイル",
                        )
                        .required(true),
                    ),
                )
                .await;

            match res {
                Ok(_) => println!("Command registered for guild {}", gid),
                Err(e) => eprintln!("Error registering command for {}: {:?}", gid, e),
            }
        }
    }
}

impl Handler {
    async fn handle_sendimage(&self, ctx: &Context, command: &CommandInteraction) {
        let path = "tmp/input.png";
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
        self.respond(ctx, command, "画像を受け取りました。解析を開始します...".to_string()).await;

        if let Err(e) = crate::download_image(&attachment.url, path).await {
            eprintln!("Error downloading image: {:?}", e);
            // notify channel about failure
            if let Err(err) = command.channel_id.say(&ctx.http, format!("画像のダウンロードに失敗しました: {:?}", e)).await {
                eprintln!("Error sending failure message: {:?}", err);
            }
            return;
        }

        let result = match crate::call_python(path) {
            Ok(r) => r,
            Err(e) => {
                // notify channel about python failure
                if let Err(err) = command.channel_id.say(&ctx.http, format!("Python実行失敗: {:?}", e)).await {
                    eprintln!("Error sending failure message: {:?}", err);
                }
                return;
            }
        };

        // send final result to the channel (can take longer than 3 seconds)
        if let Err(e) = command.channel_id.say(&ctx.http, format!("受け取った画像: {}\n分析結果: {}", attachment.url, result)).await {
            eprintln!("Error sending result message: {:?}", e);
        }
    }

    async fn respond(
        &self,
        ctx: &Context,
        command: &CommandInteraction,
        content: String,
    ) {
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content(content),
        );

        if let Err(e) = command.create_response(&ctx.http, response).await {
            eprintln!("Error sending message: {:?}", e);
        }
    }
}

pub async fn run_app() -> Result<(),Box<dyn Error>>{
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN")?;

    let intents = GatewayIntents::empty(); // スラッシュコマンドだけなら基本空でOK
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await?;

    client.start().await?;
    Ok(())
}

