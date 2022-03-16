use serenity::{
	async_trait,
	model::{
		channel::Message,
		event::MessageUpdateEvent,
	},
	prelude::*,
};

use crate::commands::ConnectFour;
use crate::rusther::EventSubHandler;

#[async_trait]
impl EventSubHandler for ConnectFour {
	async fn message(&mut self, _ctx: &Context, new_message: &Message) {
		self.dispatch(&new_message.content);
	}

	async fn message_update(&mut self, _ctx: &Context, _new_data: &MessageUpdateEvent) {
		todo!()
	}
}
