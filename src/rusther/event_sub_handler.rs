use std::sync::Arc;

#[allow(unused_imports)]
// This file exposes mutable, borrowed values from serenity::client::EventHandler method signatures
// when the serenity cache feature is disabled.
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
	async fn ready(&mut self, _ctx: Arc<Context>, _data_about_bot: Arc<Ready>) {}
	async fn message(&mut self, _ctx: Arc<Context>, _new_message: Arc<Message>) {}
	async fn message_update(&mut self, _ctx: Arc<Context>, _new_data: Arc<MessageUpdateEvent>) {}
	async fn reaction_add(&mut self, _ctx: Arc<Context>, _add_reaction: Arc<Reaction>) {}
}
