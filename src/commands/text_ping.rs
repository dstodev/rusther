use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::Message;

use rusther::MessageRecipient;

use crate::rusther;

pub struct Ping {}

#[async_trait]
impl MessageRecipient for Ping {
    async fn receive(&self, ctx: &Context, msg: &Message) {
        if msg.content == "!ping" {
            if let Err(reason) = msg.channel_id.say(&ctx.http, "Welcome!").await {
                println!("Could not send message because {}", reason);
            }
        }
    }
}
