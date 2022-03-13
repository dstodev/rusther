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

#[async_trait]
pub trait MessageRecipient: Sync + Send {
    async fn receive(&mut self, ctx: &Context, msg: &Message);
}

type CommandName = &'static str;

type TextCommandInput = Box<dyn MessageRecipient>;
type TextCommandHandler = Arc<Mutex<TextCommandInput>>;


// Real recipient
pub struct Arbiter {
    command_prefix: char,
    text_commands: HashMap<CommandName, TextCommandHandler>,
}

impl Arbiter {
    pub fn new() -> Self {
        Self {
            command_prefix: '!',
            text_commands: HashMap::new(),
        }
    }
    pub fn register_text_command(&mut self, name: CommandName, handler: TextCommandInput) -> Result<(), String> {
        let element = Arc::new(Mutex::new(handler));

        if self.text_commands.insert(name, element).is_some() {
            return Err(format!("A text command named '{}' already exists!", name));
        }
        Ok(())
    }
    fn get_command_recipient_for(&self, msg: &str) -> Option<TextCommandHandler> {
        if msg.starts_with(self.command_prefix) {
            let command_name = self.get_command_name_from(msg);
            let command = self.text_commands.get(command_name.as_str())?;

            return Some(command.clone());
        }
        None
    }
    fn get_command_name_from(&self, msg: &str) -> String {
        let mut result = String::new();

        if let Some(first_word) = msg.split(' ').next() {
            let mut s = first_word.to_string();

            if s.starts_with(self.command_prefix) {
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
        if let Some(c) = self.get_command_recipient_for(&msg.content) {
            let mut handler = c.lock().await;
            handler.receive(&ctx, &msg).await;
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is now online!", ready.user.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct UnitRecipient;

    #[async_trait]
    impl MessageRecipient for UnitRecipient {
        async fn receive(&mut self, _ctx: &Context, _msg: &Message) {}
    }

    #[test]
    fn register_text_command() {
        let mut arbiter = Arbiter::new();
        let recipient = Box::new(UnitRecipient);

        let result = arbiter.register_text_command("test", recipient);
        assert!(result.is_ok());
        assert_eq!(1, arbiter.text_commands.len());
    }

    #[test]
    fn register_text_command_duplicate() {
        let mut arbiter = Arbiter::new();

        let recipient = Box::new(UnitRecipient);
        let result = arbiter.register_text_command("test", recipient);
        assert!(result.is_ok());
        assert_eq!(1, arbiter.text_commands.len());

        let recipient = Box::new(UnitRecipient);
        let result = arbiter.register_text_command("test", recipient);
        assert!(result.is_err());
        assert_eq!(1, arbiter.text_commands.len());
    }

    #[test]
    fn get_command_name_empty() {
        let arbiter = Arbiter::new();
        let actual = arbiter.get_command_name_from("");
        assert_eq!("".to_string(), actual);
    }

    #[test]
    fn get_command_name_one_word() {
        let arbiter = Arbiter::new();
        let actual = arbiter.get_command_name_from("Hello");
        assert_eq!("Hello".to_string(), actual);
    }

    #[test]
    fn get_command_name_two_words() {
        let arbiter = Arbiter::new();
        let actual = arbiter.get_command_name_from("Hello there!");
        assert_eq!("Hello".to_string(), actual);
    }

    #[test]
    fn get_command_name_strips_prefix() {
        let arbiter = Arbiter::new();
        let input = format!("{}Hello there!", arbiter.command_prefix);
        let actual = arbiter.get_command_name_from(&input);
        assert_eq!("Hello".to_string(), actual);
    }

    #[test]
    fn get_command_recipient_for_returns_none() {
        let mut arbiter = Arbiter::new();
        let recipient = Box::new(UnitRecipient);

        let _ = arbiter.register_text_command("test", recipient);

        let input = format!("{}", arbiter.command_prefix);
        let actual = arbiter.get_command_recipient_for(&input);
        assert!(actual.is_none());
    }

    #[test]
    fn get_command_recipient_for_returns_some() {
        let mut arbiter = Arbiter::new();
        let recipient = Box::new(UnitRecipient);

        let _ = arbiter.register_text_command("test", recipient);

        let input = format!("{}test", arbiter.command_prefix);
        let actual = arbiter.get_command_recipient_for(&input);
        assert!(actual.is_some());
    }
}
