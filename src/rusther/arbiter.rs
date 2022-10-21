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

/// The shared state of the Arbiter.
/// It is defined separately than inside Arbiter; instead there, it is wrapped inside an Arc<Mutex<>>.
/// Moving the important state types out here is easier for me to understand.
struct ArbiterState {
	commands: HashMap<&'static str, Arc<Mutex<CommandHandler>>>,
	user_id: UserId,
}

impl ArbiterState {
	fn new() -> Self {
		Self {
			commands: HashMap::new(),
			user_id: UserId::default(),
		}
	}
}

/// Arbitrates events to mutable sub-event-handlers.
///
/// Arbiter is a core class which accepts Discord events using the Serenity crate. It owns event
/// input and wraps this input in Arc<> to allow sub-handlers to immutably reference them without
/// copying them.
///
/// Arbiter has authority over how a Discord event is dispatched to sub-handlers. It moves
/// data to fit EventSubHandler trait function definitions.
///
/// For every Discord event to be supported, an analogue Arbiter method must exist. Its role
/// is to dispatch each sub-handler, ideally as individual tasks.
///
/// A foundational ability of Arbiter is to provide mutability to event sub-handlers, which is
/// especially useful for interactions between events over time.
/// This works by providing interior mutability via the Arc<Mutex<>> type.
///
/// Atomically-reference-counted (ARC) pointer clones for each sub-handler are distributed to
/// tasks per event invocation, each pointing to the mutex guarding the event handler. Tasks lock
/// the sub-handler mutex then execute them, passing above-described clones of inputs as their
/// like-typed Arc<>s.
///
/// The ready() function serves as a notable example of implementing one such "event forwarder".
///
/// PERFORMANCE WARNING:
///     All Discord events handled by a CommandHandler are processed by the same instance, locking
///     it each time. Care should be taken within each handler to return as soon as possible,
///     with performance-heavy code being spawned in new tasks instead of executing in the handler
///     itself.
pub struct Arbiter {
	command_prefix: char,
	state: Arc<Mutex<ArbiterState>>,
}

impl Arbiter {
	pub fn new() -> Self {
		Self {
			command_prefix: '!',
			state: Arc::new(Mutex::new(ArbiterState::new())),
		}
	}
	// TODO: Make providing a name optional
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
		if msg.author.id == ctx.cache.current_user_id() {
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
			if user.id == ctx.cache.current_user_id() {
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
				if user_id == safe_ctx.cache.current_user_id() {
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

		// Clones of these Arc<> will be moved to child tasks.
		let safe_ctx = Arc::new(ctx);
		let safe_ready = Arc::new(ready);

		let handlers;
		{
			/* self.commands is a list of Arc<>, so cloning it is not cloning the handlers.
			   Instead, it is cloning pointers to the handlers, which are locked behind mutexes.

			   Access to the state mutex is kept minimal and it is released as soon as possible
			   by dropping from scope.
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
