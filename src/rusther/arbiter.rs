use std::collections::HashMap;
use std::sync::Arc;

use serenity::{
	async_trait,
	model::{
		channel::Message,
		gateway::Ready,
	},
	prelude::*,
};
use tokio::sync::Mutex;

use crate::rusther::EventSubHandler;

type CommandHandler = Box<dyn EventSubHandler>;
type SafeCommandHandler = Arc<Mutex<CommandHandler>>;


// Real recipient
pub struct Arbiter {
	command_prefix: char,
	commands: HashMap<&'static str, SafeCommandHandler>,
}

impl Arbiter {
	pub fn new() -> Self {
		Self {
			command_prefix: '!',
			commands: HashMap::new(),
		}
	}
	pub fn register_event_handler(&mut self, name: &'static str, handler: CommandHandler) -> Result<(), String> {
		let element = Arc::new(Mutex::new(handler));

		if self.commands.insert(name, element).is_some() {
			return Err(format!("A command named '{}' already exists!", name));
		}
		Ok(())
	}
	fn get_command_handler_for(&self, name: &'static str) -> Option<&SafeCommandHandler> {
		self.commands.get(name)
	}
	fn get_command_name_from(prefix: char, msg: &str) -> String {
		let mut result = String::new();

		if let Some(first_word) = msg.split(' ').next() {
			let mut s = first_word.to_string();

			if s.starts_with(prefix) {
				s.remove(0);
			}
			result = s;
		}
		result
	}
}

#[async_trait]
impl EventHandler for Arbiter {
	async fn message(&self, ctx: Context, msg: Message) {
		if msg.content.starts_with(self.command_prefix) {
			// Strip the command prefix from the message
			let mut message = msg;
			message.content.remove(0);

			for (_k, v) in self.commands.iter() {
				let mut handler = v.lock().await;
				handler.message(&ctx, &message).await;
			}
		}
	}
	async fn ready(&self, ctx: Context, ready: Ready) {
		// This should remain the simplest event forwarder, as example.
		for (_k, v) in self.commands.iter() {
			let mut handler = v.lock().await;
			handler.ready(&ctx, &ready).await;
		}
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
		assert_eq!(1, arbiter.commands.len());
	}

	#[test]
	fn register_text_command_duplicate() {
		let mut arbiter = Arbiter::new();

		let recipient = Box::new(UnitRecipient);
		let result = arbiter.register_event_handler("test", recipient);
		assert!(result.is_ok());
		assert_eq!(1, arbiter.commands.len());

		let recipient = Box::new(UnitRecipient);
		let result = arbiter.register_event_handler("test", recipient);
		assert!(result.is_err());
		assert_eq!(1, arbiter.commands.len());
	}

	#[test]
	fn get_command_name_from_empty() {
		let actual = Arbiter::get_command_name_from('!', "");
		assert_eq!("".to_string(), actual);
	}

	#[test]
	fn get_command_name_from_one_word() {
		let actual = Arbiter::get_command_name_from('!', "Hello");
		assert_eq!("Hello".to_string(), actual);
	}

	#[test]
	fn get_command_name_from_two_words() {
		let actual = Arbiter::get_command_name_from('!', "Hello there!");
		assert_eq!("Hello".to_string(), actual);
	}

	#[test]
	fn get_command_name_from_strips_prefix() {
		let arbiter = Arbiter::new();
		let input = format!("{}Hello there!", arbiter.command_prefix);
		let actual = Arbiter::get_command_name_from('!', &input);
		assert_eq!("Hello".to_string(), actual);
	}

	#[test]
	fn get_command_handler_for_returns_none() {
		let mut arbiter = Arbiter::new();
		let recipient = Box::new(UnitRecipient);

		let _ = arbiter.register_event_handler("test", recipient);
		let actual = arbiter.get_command_handler_for("");
		assert!(actual.is_none());
	}

	#[test]
	fn get_command_handler_for_returns_some() {
		let mut arbiter = Arbiter::new();
		let recipient = Box::new(UnitRecipient);

		let _ = arbiter.register_event_handler("test", recipient);
		let actual = arbiter.get_command_handler_for("test");
		assert!(actual.is_some());
	}
}
