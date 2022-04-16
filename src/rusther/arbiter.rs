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

struct InstanceState {
	commands: HashMap<&'static str, CommandHandler>,
	user_id: UserId,
}

impl InstanceState {
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
	state: Arc<Mutex<InstanceState>>,
}

impl Arbiter {
	pub fn new() -> Self {
		Self {
			command_prefix: '!',
			state: Arc::new(Mutex::new(InstanceState::new())),

		}
	}
	pub fn register_event_handler(&mut self, name: &'static str, handler: CommandHandler) -> Result<(), String> {
		let mut state = self.state.blocking_lock();

		if state.commands.insert(name, handler).is_some() {
			return Err(format!("A command named '{}' already exists!", name));
		}
		Ok(())
	}
	fn sanitize(content: String) -> String {
		let mut result = content;

		// Strip the command prefix from the message
		result.remove(0);

		result
	}
}

#[async_trait]
impl EventHandler for Arbiter {
	async fn message(&self, ctx: Context, mut msg: Message) {
		let mut state = self.state.lock().await;

		if msg.author.id == state.user_id {
			log::trace!("Skipping own message");
			return;
		}
		if msg.content.starts_with(self.command_prefix) {
			msg.content = Self::sanitize(msg.content);

			join_all(state
				.commands
				.values_mut()
				.map(|handler| handler.message(&ctx, &msg))
			).await;
		}
	}
	async fn message_update(&self, ctx: Context, _old_if_available: Option<Message>, _new: Option<Message>, event: MessageUpdateEvent) {
		let mut state = self.state.lock().await;

		if let Some(user) = &event.author {
			if user.id == state.user_id {
				log::trace!("Skipping own message_update");
				return;
			}
		}
		join_all(state
			.commands
			.values_mut()
			.map(|handler| handler.message_update(&ctx, &event))
		).await;
	}
	async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
		let mut state = self.state.lock().await;

		if let Ok(user) = add_reaction.user(&ctx.http).await {
			if user.id == state.user_id {
				log::trace!("Skipping own reaction_add");
				return;
			}
		}
		join_all(state
			.commands
			.values_mut()
			.map(|handler| handler.reaction_add(&ctx, &add_reaction))
		).await;
	}
	async fn ready(&self, ctx: Context, ready: Ready) {
		// This function should remain the simplest event forwarder, as example.
		let mut state = self.state.lock().await;

		state.user_id = ready.user.id;  // Store bot instance's id for later comparisons

		join_all(state
			.commands
			.values_mut()
			.map(|handler| handler.ready(&ctx, &ready))
		).await;
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

	#[test]
	fn sanitize_simple_message() {
		let input = "!lorem ipsum".to_string();
		let actual = Arbiter::sanitize(input);
		assert_eq!("lorem ipsum", &actual);
	}
}
