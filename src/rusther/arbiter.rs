use std::collections::HashMap;
use std::sync::Arc;

use serenity::{
	async_trait,
	futures::future::join_all,
	model::{
		channel::{
			Message,
			Reaction,
		},
		event::MessageUpdateEvent,
		gateway::Ready,
		id::UserId,
	},
	prelude::*,
};
use tokio::sync::Mutex;

use crate::rusther::EventSubHandler;

type CommandHandler = Box<dyn EventSubHandler>;

struct State {
	commands: HashMap<&'static str, CommandHandler>,
	user_id: UserId,
}

impl State {
	fn new() -> Self {
		Self {
			commands: HashMap::new(),
			user_id: UserId::default(),
		}
	}
}

// Real recipient
pub struct Arbiter {
	command_prefix: char,
	state: Arc<Mutex<State>>,
}

impl Arbiter {
	pub fn new() -> Self {
		Self {
			command_prefix: '!',
			state: Arc::new(Mutex::new(State::new())),

		}
	}
	pub fn register_event_handler(&mut self, name: &'static str, handler: CommandHandler) -> Result<(), String> {
		let mut state = self.state.blocking_lock();

		if state.commands.insert(name, handler).is_some() {
			return Err(format!("A command named '{}' already exists!", name));
		}
		Ok(())
	}
}

#[async_trait]
impl EventHandler for Arbiter {
	async fn message(&self, ctx: Context, msg: Message) {
		let mut state = self.state.lock().await;

		if msg.author.id == state.user_id {
			println!("Skipping own message");
			return;
		}
		if msg.content.starts_with(self.command_prefix) {
			// Strip the command prefix from the message
			let mut message = msg;
			message.content.remove(0);

			let mut futures = Vec::new();

			for (_name, handler) in state.commands.iter_mut() {
				futures.push(handler.message(&ctx, &message))
			}
			join_all(futures).await;
		}
	}

	async fn message_update(&self, ctx: Context, new_data: MessageUpdateEvent) {
		let mut state = self.state.lock().await;

		if let Some(user) = &new_data.author {
			if user.id == state.user_id {
				println!("Skipping own message_update");
				return;
			}
		}
		let mut futures = Vec::new();

		for (_name, handler) in state.commands.iter_mut() {
			futures.push(handler.message_update(&ctx, &new_data));
		}
		join_all(futures).await;
	}

	async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
		let mut state = self.state.lock().await;

		if let Ok(user) = add_reaction.user(&ctx.http).await {
			if user.id == state.user_id {
				println!("Skipping own reaction_add");
				return;
			}
		}
		let mut futures = Vec::new();

		for (_name, handler) in state.commands.iter_mut() {
			futures.push(handler.reaction_add(&ctx, &add_reaction));
		}
		join_all(futures).await;
	}

	async fn ready(&self, ctx: Context, ready: Ready) {
		// This function should remain the simplest event forwarder, as example.
		let mut state = self.state.lock().await;

		state.user_id = ready.user.id;  // Store bot instance's id for later comparisons

		let mut futures = Vec::new();

		for (_name, handler) in state.commands.iter_mut() {
			futures.push(handler.ready(&ctx, &ready));
		}
		join_all(futures).await;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct UnitRecipient;

	#[async_trait]
	impl EventSubHandler for UnitRecipient {
		async fn message(&mut self, _ctx: &Context, _msg: &Message) {}
	}

	#[test]
	fn register_text_command() {
		let mut arbiter = Arbiter::new();
		let recipient = Box::new(UnitRecipient);

		let result = arbiter.register_event_handler("test", recipient);
		assert!(result.is_ok());
	}

	#[test]
	fn register_text_command_duplicate() {
		let mut arbiter = Arbiter::new();

		let recipient = Box::new(UnitRecipient);
		let result = arbiter.register_event_handler("test", recipient);
		assert!(result.is_ok());

		let recipient = Box::new(UnitRecipient);
		let result = arbiter.register_event_handler("test", recipient);
		assert!(result.is_err());
	}
}
