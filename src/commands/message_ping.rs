use serenity::{
    async_trait,
    model::channel::Message,
    prelude::*,
};

use crate::rusther::EventSubHandler;

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
impl EventSubHandler for Ping {
    async fn message(&mut self, ctx: &Context, msg: &Message) {
        if msg.content == "!ping" {
            self.value += 1;
            let message = format!("Welcome #{}!", self.value);

            if let Err(reason) = msg.channel_id.say(&ctx.http, message).await {
                println!("Could not send message because {}", reason);
            }
        }
    }
}
