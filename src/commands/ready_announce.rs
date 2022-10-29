use serenity::{async_trait, model::gateway::Ready, prelude::*};

use crate::rusther::EventSubHandler;

pub struct Announce;

#[async_trait]
impl EventSubHandler for Announce {
    async fn ready(&mut self, _context: Context, data_about_bot: Ready) {
        log::info!("{} is now online!", data_about_bot.user.name);
    }
}
