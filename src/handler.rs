use serenity::{async_trait, model::prelude::*, prelude::*};

use crate::commands;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        match command.data.name.as_str() {
            commands::COMMAND_NAME => commands::handle_is_this_kiwi(&ctx, &command).await,
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, _: Ready) {
        // コマンドの登録先を決める。環境変数 GUILD_ID が数値として設定されていれば
        // そのギルドのみ、そうでなければ参加済みの全ギルドに登録する。
        let guild_ids: Vec<GuildId> = match std::env::var("GUILD_ID") {
            Ok(s) => match s.parse::<u64>() {
                Ok(n) => vec![GuildId::new(n)],
                Err(_) => ctx.cache.guilds().iter().cloned().collect(),
            },
            Err(_) => ctx.cache.guilds().iter().cloned().collect(),
        };

        for gid in guild_ids {
            let res = gid
                .create_command(&ctx.http, commands::create_command())
                .await;

            match res {
                Ok(_) => println!("Command registered for guild {}", gid),
                Err(e) => eprintln!("Error registering command for {}: {:?}", gid, e),
            }
        }
    }
}
