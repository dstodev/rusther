use std::collections::HashMap;
use std::sync::Arc;

use serenity::{
	async_trait,
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
use tokio::{
	sync::Mutex,
	task::yield_now,
};

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
		let mutex = self.state.clone().lock_owned();
		let prefix = self.command_prefix;

		tokio::spawn(async move {
			let mut state = mutex.await;

			if msg.author.id == state.user_id {
				log::trace!("Skipping own message");
				return;
			}
			if msg.content.starts_with(prefix) {
				msg.content = Self::sanitize(msg.content);

				for handler in state.commands.values_mut() {
					handler.message(&ctx, &msg).await;
					yield_now().await;
				}
			}
		});
	}
	async fn message_update(&self, ctx: Context, _old_if_available: Option<Message>, _new: Option<Message>, event: MessageUpdateEvent) {
		let mutex = self.state.clone().lock_owned();

		tokio::spawn(async move {
			let mut state = mutex.await;

			if let Some(user) = &event.author {
				if user.id == state.user_id {
					log::trace!("Skipping own message_update");
					return;
				}
			}
			for handler in state.commands.values_mut() {
				handler.message_update(&ctx, &event).await;
				yield_now().await;
			}
		});
	}
	async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
		let mutex = self.state.clone().lock_owned();

		tokio::spawn(async move {
			let mut state = mutex.await;

			if let Ok(user) = add_reaction.user(&ctx.http).await {
				if user.id == state.user_id {
					log::trace!("Skipping own reaction_add");
					return;
				}
			}
			for handler in state.commands.values_mut() {
				handler.reaction_add(&ctx, &add_reaction).await;
				yield_now().await;
			}
		});
	}
	async fn ready(&self, ctx: Context, ready: Ready) {
		// This function should remain the simplest event forwarder, as example.

		// self.state is an Arc<>, so cloning it is not "cloning" the context, per-se. Instead,
		// it is cloning the pointer to the state, which is locked behind a mutex.
		let mutex = self.state.clone().lock_owned();

		tokio::spawn(async move {
			// The task will lock the state and work through the event sub-handlers one-at-a-time
			// until the input has fully propagated.
			let mut state = mutex.await;

			state.user_id = ready.user.id;  // Store bot instance's id for later comparisons

			for handler in state.commands.values_mut() {
				handler.ready(&ctx, &ready).await;

				// The task should yield after each sub-handler completes, to give other tasks time
				// to process as well.
				yield_now().await;
			}
		});
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
