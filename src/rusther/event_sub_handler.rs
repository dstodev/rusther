// This file exposes mutable, borrowed values from serenity::client::EventHandler method signatures

use serenity::{
	async_trait,
	model::{
		channel::Message,
		event::MessageUpdateEvent,
		gateway::Ready,
	},
	prelude::*,
};

#[async_trait]
pub trait EventSubHandler: Sync + Send {
	// async fn message(&self, _ctx: Context, _new_message: Message) {}
	async fn message(&mut self, _ctx: &Context, _new_message: &Message) {}

	// async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {}
	async fn ready(&mut self, _ctx: &Context, _data_about_bot: &Ready) {}

	// async fn message_update(&self, _ctx: Context, _new_data: MessageUpdateEvent) {}
	async fn message_update(&mut self, _ctx: &Context, _new_data: &MessageUpdateEvent) {}
}
