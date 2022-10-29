use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction},
        event::MessageUpdateEvent,
        gateway::Ready,
    },
    prelude::*,
};
use tokio::{runtime::Handle, sync::broadcast};

use crate::rusther::EventSubHandler;

/// Arbitrates events to mutable event-(sub)-handlers.
///
/// Arbiter is a core class which accepts Discord events using the Serenity crate.
///
/// Arbiter has authority over how a Discord event is dispatched to sub-handlers. It moves
/// data to fit EventSubHandler trait function definitions.
///
/// For every Discord event to be supported, an analogue Arbiter method must exist. Its role
/// is to dispatch arguments to each sub-handler, which ideally are individual tasks.
///
/// A foundational ability of Arbiter is to provide mutability to event sub-handlers, which is
/// especially useful for interactions between events over time.
pub struct Arbiter {
    tokio_rt_handle: Handle,
    command_prefix: char,

    message_tx: Option<broadcast::Sender<(Context, Message)>>,
    message_update_tx: Option<
        broadcast::Sender<(
            Context,
            Option<Message>,
            Option<Message>,
            MessageUpdateEvent,
        )>,
    >,
    reaction_add_tx: Option<broadcast::Sender<(Context, Reaction)>>,
    ready_tx: Option<broadcast::Sender<(Context, Ready)>>,
}

impl Arbiter {
    pub fn new(handle: Handle) -> Self {
        const CHANNEL_CAPACITY: usize = 100;
        const PREFIX: char = '!';

        let (message_tx, _message_rx) = broadcast::channel(CHANNEL_CAPACITY);
        let (message_update_tx, _message_update_rx) = broadcast::channel(CHANNEL_CAPACITY);
        let (reaction_add_tx, _reaction_add_rx) = broadcast::channel(CHANNEL_CAPACITY);
        let (ready_tx, _ready_rx) = broadcast::channel(CHANNEL_CAPACITY);

        Self {
            tokio_rt_handle: handle,
            command_prefix: PREFIX,

            message_tx: Some(message_tx),
            message_update_tx: Some(message_update_tx),
            reaction_add_tx: Some(reaction_add_tx),
            ready_tx: Some(ready_tx),
        }
    }
    // TODO: Make providing a name optional
    pub fn register_event_handler(
        &mut self,
        handler: impl EventSubHandler + 'static,
    ) -> Result<(), String> {
        let mut message_rx = self.message_tx.as_ref().unwrap().subscribe();
        let mut message_update_rx = self.message_update_tx.as_ref().unwrap().subscribe();
        let mut reaction_add_rx = self.reaction_add_tx.as_ref().unwrap().subscribe();
        let mut ready_rx = self.ready_tx.as_ref().unwrap().subscribe();

        self.tokio_rt_handle.spawn(async move {
            let mut handler = handler;
            loop {
                tokio::select! {
                    Ok((context, message)) = message_rx.recv() => handler.message(context, message).await,
                    Ok((context, old, new, event)) = message_update_rx.recv() => handler.message_update(context, old, new, event).await,
                    Ok((context, reaction)) = reaction_add_rx.recv() => handler.reaction_add(context, reaction).await,
                    Ok((context, ready)) = ready_rx.recv() => handler.ready(context, ready).await,
                    else => break,
                }
            }
        });

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
    async fn message(&self, context: Context, mut msg: Message) {
        if msg.author.id == context.cache.current_user_id() {
            log::trace!("Skipping own message");
            return;
        }
        let prefix = self.command_prefix;

        if let Some(message_tx) = &self.message_tx {
            if msg.content.starts_with(prefix) {
                msg.content = Self::sanitize(msg.content);
                let _ = message_tx.send((context, msg));
            }
        }
    }
    async fn message_update(
        &self,
        context: Context,
        old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if let Some(user) = &event.author {
            if user.id == context.cache.current_user_id() {
                log::trace!("Skipping own message_update");
                return;
            }
        }
        if let Some(message_update_tx) = &self.message_update_tx {
            let _ = message_update_tx.send((context, old, new, event));
        }
    }
    async fn reaction_add(&self, context: Context, reaction: Reaction) {
        if let Some(user_id) = reaction.user_id {
            if user_id == context.cache.current_user_id() {
                log::trace!("Skipping own reaction_add");
                return;
            }
        }
        if let Some(reaction_add_tx) = &self.reaction_add_tx {
            let _ = reaction_add_tx.send((context, reaction));
        }
    }
    async fn ready(&self, context: Context, ready: Ready) {
        if let Some(ready_tx) = &self.ready_tx {
            let _ = ready_tx.send((context, ready));
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::runtime::Runtime;

    use super::*;

    struct UnitRecipient;

    #[async_trait]
    impl EventSubHandler for UnitRecipient {
        async fn message(&mut self, _context: Context, _msg: Message) {}
    }

    #[test]
    fn register_text_command() {
        let rt = Runtime::new().unwrap();
        let mut arbiter = Arbiter::new(rt.handle().clone());

        let result = arbiter.register_event_handler(UnitRecipient);
        assert!(result.is_ok());
    }

    #[test]
    fn sanitize_simple_message() {
        let input = "!lorem ipsum".to_string();
        let actual = Arbiter::sanitize(input);
        assert_eq!("lorem ipsum", &actual);
    }
}
