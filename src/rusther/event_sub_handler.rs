#[allow(unused_imports)]
use serenity::{
    async_trait,
    model::{channel::Message, channel::Reaction, event::MessageUpdateEvent, gateway::Ready},
    prelude::*,
};

#[async_trait]
pub trait EventSubHandler: Sync + Send {
    async fn ready(&mut self, _context: Context, _data_about_bot: Ready) {}
    async fn message(&mut self, _context: Context, _message: Message) {}
    async fn message_update(
        &mut self,
        _context: Context,
        _old: Option<Message>,
        _new: Option<Message>,
        _message_update: MessageUpdateEvent,
    ) {
    }
    async fn reaction_add(&mut self, _context: Context, _reaction: Reaction) {}
}
