#[allow(unused_imports)]
// This file exposes mutable, borrowed values from serenity::client::EventHandler method signatures
use serenity::client::EventHandler;  // Link for convenience

use serenity::{
	async_trait,
	model::{
		channel::Message,
		event::MessageUpdateEvent,
		channel::Reaction,
		gateway::Ready,
	},
	prelude::*,
};

#[async_trait]
pub trait EventSubHandler: Sync + Send {
	// async fn ready(&self, _ctx: Context, _data_about_bot: Ready) {}
	async fn ready(&mut self, _ctx: &Context, _data_about_bot: &Ready) {}

	// async fn message(&self, _ctx: Context, _new_message: Message) {}
	async fn message(&mut self, _ctx: &Context, _new_message: &Message) {}

	// async fn message_update(&self, _ctx: Context, _new_data: MessageUpdateEvent) {}
	async fn message_update(&mut self, _ctx: &Context, _new_data: &MessageUpdateEvent) {}

	// async fn reaction_add(&self, _ctx: Context, _add_reaction: Reaction) {}
	async fn reaction_add(&mut self, _ctx: &Context, _add_reaction: &Reaction) {}
}
