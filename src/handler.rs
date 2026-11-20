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
        // Register the command. If `GUILD_ID` env var is set to a numeric guild id,
        // register only there. Otherwise register the command in every guild the
        // bot is cached in (so it's available on all servers the bot has joined).
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
