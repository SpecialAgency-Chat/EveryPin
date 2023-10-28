use anyhow::anyhow;
use serenity::model::prelude::ReactionType;
use serenity::{async_trait, model::prelude::Reaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_secrets::SecretStore;
use tracing::{error, info, trace};

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        trace!("reaction_add: {:?}", reaction);
        let channel = reaction.channel_id.to_channel(&ctx).await.unwrap();
        if channel.clone().private().is_some() {
            return;
        }
        let channel = channel.guild().unwrap();
        if reaction.emoji.unicode_eq("📌") && channel.topic.unwrap_or("".into()).contains("Pin") {
            let message = reaction.message(&ctx).await.unwrap();
            let result = message.pin(&ctx).await;
            if let Err(e) = result {
                error!("Failed to pin message: {:?}", e);
                message.react(&ctx, ReactionType::Unicode("❌".into())).await.ok();
            }
        }
    }
    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        trace!("reaction_remove: {:?}", reaction);
        let channel = reaction.channel_id.to_channel(&ctx).await.unwrap();
        if channel.clone().private().is_some() {
            return;
        }
        let channel = channel.guild().unwrap();
        if reaction.emoji.unicode_eq("📌") && channel.topic.unwrap_or("".into()).contains("Pin") {
            let message = reaction.message(&ctx).await.unwrap();
            if message.reactions.iter().find(|r| {
                if let ReactionType::Unicode(s) = &r.reaction_type {
                    s == "📌"
                } else {
                    false
                }
            }).is_none() {
                let result = message.unpin(&ctx).await;
                if let Err(e) = result {
                    error!("Failed to unpin message: {:?}", e);
                    message.react(&ctx, ReactionType::Unicode("❌".into())).await.ok();
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = if let Some(token) = secret_store.get("DISCORD_TOKEN") {
        token
    } else {
        return Err(anyhow!("'DISCORD_TOKEN' was not found").into());
    };

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    let client = Client::builder(&token, intents)
        .event_handler(Bot)
        .await
        .expect("Err creating client");

    Ok(client.into())
}
