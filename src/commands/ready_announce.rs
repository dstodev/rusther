use serenity::{
	async_trait,
	model::gateway::Ready,
	prelude::*,
};

use crate::rusther::EventSubHandler;

pub struct Announce;

#[async_trait]
impl EventSubHandler for Announce {
	async fn ready(&mut self, _ctx: &Context, data_about_bot: &Ready) {
		println!("{} is now online!", data_about_bot.user.name);
	}
}
