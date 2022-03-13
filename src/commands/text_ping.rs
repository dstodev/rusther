use serenity::async_trait;
use serenity::client::Context;
use serenity::model::channel::Message;

use rusther::MessageRecipient;

use crate::rusther;

pub struct Ping {
    value: i32,
}

impl Ping {
    pub fn new() -> Self {
        Self {
            value: 0,
        }
    }
}

#[async_trait]
impl MessageRecipient for Ping {
    async fn receive(&mut self, ctx: &Context, msg: &Message) {
        if msg.content == "!ping" {
            self.value += 1;
            let message = format!("Welcome #{}!", self.value);

            if let Err(reason) = msg.channel_id.say(&ctx.http, message).await {
                println!("Could not send message because {}", reason);
            }
        }
    }
}
