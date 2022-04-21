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
use tokio::sync::Mutex;

use crate::rusther::EventSubHandler;

type CommandHandler = Box<dyn EventSubHandler>;

struct InstanceState {
	commands: HashMap<&'static str, Arc<Mutex<CommandHandler>>>,
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

		if state.commands
		        .insert(name, Arc::new(Mutex::new(handler)))
		        .is_some() {
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
	async fn copy_handler_pointers(&self) -> Vec<Arc<Mutex<CommandHandler>>> {
		let state = self.state.lock().await;
		Vec::from_iter(state.commands.values().cloned())
	}
}

#[async_trait]
impl EventHandler for Arbiter {
	async fn message(&self, ctx: Context, mut msg: Message) {
		if msg.author.id == ctx.cache.current_user_id().await {
			log::trace!("Skipping own message");
			return;
		}
		let prefix = self.command_prefix;

		if msg.content.starts_with(prefix) {
			msg.content = Self::sanitize(msg.content);

			let safe_ctx = Arc::new(ctx);
			let safe_msg = Arc::new(msg);

			let handlers = self.copy_handler_pointers().await;

			for handler in handlers {
				let sub_ctx = safe_ctx.clone();
				let sub_msg = safe_msg.clone();

				tokio::spawn(async move {
					let mut lock = handler.lock().await;
					lock.message(sub_ctx, sub_msg).await;
				});
			}
		}
	}
	async fn message_update(&self, ctx: Context, _old_if_available: Option<Message>, _new: Option<Message>, event: MessageUpdateEvent) {
		if let Some(user) = &event.author {
			if user.id == ctx.cache.current_user_id().await {
				log::trace!("Skipping own message_update");
				return;
			}
		}
		let safe_ctx = Arc::new(ctx);
		let safe_event = Arc::new(event);

		let handlers = self.copy_handler_pointers().await;

		for handler in handlers {
			let sub_ctx = safe_ctx.clone();
			let sub_event = safe_event.clone();

			tokio::spawn(async move {
				let mut lock = handler.lock().await;
				lock.message_update(sub_ctx, sub_event).await;
			});
		}
	}
	async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
		let handlers = self.copy_handler_pointers().await;

		tokio::spawn(async move {
			let safe_ctx = Arc::new(ctx);
			let safe_add_reaction = Arc::new(add_reaction);

			if let Some(user_id) = safe_add_reaction.user_id {
				if user_id == safe_ctx.cache.current_user_id().await {
					log::trace!("Skipping own reaction_add");
					return;
				}
			}

			for handler in handlers {
				let sub_ctx = safe_ctx.clone();
				let sub_add_reaction = safe_add_reaction.clone();

				tokio::spawn(async move {
					let mut lock = handler.lock().await;
					lock.reaction_add(sub_ctx, sub_add_reaction).await;
				});
			}
		});
	}
	async fn ready(&self, ctx: Context, ready: Ready) {
		// This function should remain the simplest event forwarder, as example.

		// Wrap inputs--Arc<> clones will be moved to child tasks.
		let safe_ctx = Arc::new(ctx);
		let safe_ready = Arc::new(ready);

		let handlers;
		{
			/* self.state is an Arc<>, so cloning it is not cloning the state, per-se. Instead,
			     it is cloning the pointer to the state, which is additionally locked behind a
			     mutex.
			   Access to the state is kept minimal, with mutexes being unlocked (e.g. dropped
			     from scope) as soon as possible.
			*/
			let mut state = self.state.lock().await;
			state.user_id = safe_ready.user.id;  // Store bot instance's id for later comparisons
			handlers = Vec::from_iter(state.commands.values().cloned());
		}
		// Spawn a task for each sub-handler.
		for handler in handlers {
			let sub_ctx = safe_ctx.clone();
			let sub_ready = safe_ready.clone();

			tokio::spawn(async move {
				// Each task will wait for and then give input to a sub-handler.
				let mut lock = handler.lock().await;
				lock.ready(sub_ctx, sub_ready).await;
			});
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct UnitRecipient;

	#[async_trait]
	impl EventSubHandler for UnitRecipient {
		async fn message(&mut self, _ctx: Arc<Context>, _msg: Arc<Message>) {}
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
