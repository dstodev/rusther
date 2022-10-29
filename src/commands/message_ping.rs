use serenity::{async_trait, model::channel::Message, prelude::*};

use crate::rusther::EventSubHandler;

pub struct Ping {
    value: i32,
}

impl Ping {
    pub fn new() -> Self {
        Self { value: 0 }
    }
}

#[async_trait]
impl EventSubHandler for Ping {
    async fn message(&mut self, context: Context, msg: Message) {
        match msg.content.as_str() {
            "ping" | "hello" | "welcome" => {
                self.value += 1;
                let say = format!("Welcome #{}!", self.value);

                if let Err(reason) = msg.channel_id.say(&context.http, say).await {
                    log::debug!("Could not send message because {}", reason);
                }
            }
            _ => {}
        }
    }
}
