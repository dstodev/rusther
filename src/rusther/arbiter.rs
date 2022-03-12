use collections::HashMap;
use std::collections;

use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
    },
    prelude::*,
};

#[async_trait]
pub trait MessageRecipient: Sync + Send {
    async fn receive(&self, ctx: &Context, msg: &Message);
}

pub struct Arbiter {
    prefix: char,
    text_commands: HashMap<&'static str, Box<dyn MessageRecipient>>,
}

impl Arbiter {
    pub fn new() -> Self {
        Self {
            prefix: '!',
            text_commands: HashMap::new(),
        }
    }
    pub fn register_text_command(&mut self, name: &'static str, handler: Box<dyn MessageRecipient>) {
        if self.text_commands.insert(name, handler).is_some() {
            panic!("A text command named '{}' already exists!", name);
        }
    }
    fn get_command_name(&self, msg: &String) -> String {
        if let Some(first_word) = msg.split(' ').next() {
            let mut s = String::from(first_word);
            if s.starts_with(self.prefix) {
                s.remove(0);
            }
            return s;
        }
        String::new()
    }
}

#[async_trait]
impl EventHandler for Arbiter {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.starts_with(self.prefix) {
            let command_name = self.get_command_name(&msg.content);

            if let Some(c) = self.text_commands.get(command_name.as_str()) {
                c.receive(&ctx, &msg).await;
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} now online!", ready.user.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_command_name_empty() {
        let arbiter = Arbiter::new();
        let input = String::from("");
        let actual = arbiter.get_command_name(&input);
        assert_eq!(input, actual);
    }

    #[test]
    fn get_command_name_one_word() {
        let arbiter = Arbiter::new();
        let input = String::from("hello");
        let actual = arbiter.get_command_name(&input);
        assert_eq!(input, actual);
    }

    #[test]
    fn get_command_name_two_words() {
        let arbiter = Arbiter::new();
        let input = String::from("hello there!");
        let actual = arbiter.get_command_name(&input);
        assert_eq!(String::from("hello"), actual);
    }

    #[test]
    fn get_command_strips_prefix() {
        let arbiter = Arbiter::new();
        let input = format!("{}hello there!", arbiter.prefix);
        let actual = arbiter.get_command_name(&input);
        assert_eq!(String::from("hello"), actual);
    }
}
