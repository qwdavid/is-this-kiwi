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
            "sendimage" => self.handle_sendimage(&ctx, &command).await,
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        let result = GuildId::new(1373513874497601658)
            .create_command(
                &ctx.http,
                CreateCommand::new("sendimage")
                    .description("画像を送信するコマンド")
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

        match result {
            Ok(_) => println!("Command registered successfully"),
            Err(e) => eprintln!("Error registering command: {:?}", e),
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

        if let Err(e) = crate::download_image(&attachment.url, path).await {
            eprintln!("Error downloading image: {:?}", e);
            return;
        }

        let result = match crate::call_python(path) {
            Ok(r) => r,
            Err(e) => {
                self.respond(ctx, command, format!("Python実行失敗: {:?}", e)).await;
                return;
            }
        };

        self.respond(
            ctx,
            command,
            format!("受け取った画像: {}\n分析結果: {}", attachment.url, result),
        )
        .await;
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

