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
}

#[async_trait]
impl EventHandler for Arbiter {
	async fn message(&self, ctx: Context, mut msg: Message) {
		let state = self.state.lock().await;
		let prefix = self.command_prefix;

		if msg.author.id == state.user_id {
			log::trace!("Skipping own message");
			return;
		}
		if msg.content.starts_with(prefix) {
			msg.content = Self::sanitize(msg.content);

			let safe_ctx = Arc::new(ctx);
			let safe_msg = Arc::new(msg);

			for handler in state.commands.values().cloned() {
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
		// This is a simpler form of acquiring the state,
		// and it should be used when further tasks are not given the state.
		let state = self.state.lock().await;

		// This kind of setup does not need to be awaited, so no need for a wrapper task.
		if let Some(user) = &event.author {
			if user.id == state.user_id {
				log::trace!("Skipping own message_update");
				return;
			}
		}
		let safe_ctx = Arc::new(ctx);
		let safe_event = Arc::new(event);

		for handler in state.commands.values().cloned() {
			let sub_ctx = safe_ctx.clone();
			let sub_event = safe_event.clone();

			tokio::spawn(async move {
				let mut lock = handler.lock().await;
				lock.message_update(sub_ctx, sub_event).await;
			});
		}
	}
	async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
		/* .clone().lock_owned() should be used when the setup takes time, e.g. waiting for
		   a reference to a user; when the role of dispatching tasks becomes a task in itself.
		   This is because it allows the state pointer to be moved to a task.

		   self.state is an Arc<>, so cloning it is not "cloning" the context, per-se. Instead,
		   it is cloning the pointer to the state, which is locked behind a mutex.
		*/
		let state = self.state.clone().lock_owned().await;

		let safe_ctx = Arc::new(ctx);
		let safe_add_reaction = Arc::new(add_reaction);

		tokio::spawn(async move {
			if let Some(user_id) = safe_add_reaction.user_id {
				if user_id == safe_ctx.cache.current_user_id().await {
					log::trace!("Skipping own reaction_add");
					return;
				}
			}
			for handler in state.commands.values().cloned() {
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
		let mut state = self.state.lock().await;

		state.user_id = ready.user.id;  // Store bot instance's id for later comparisons

		// Wrap inputs--pointer clones will be moved to further tasks.
		let safe_ctx = Arc::new(ctx);
		let safe_ready = Arc::new(ready);

		// Spawn a task for each sub-handler.
		for handler in state.commands.values().cloned() {
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
