use std::collections::HashMap;
use std::sync::Arc;

use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction},
        event::MessageUpdateEvent,
        gateway::Ready,
        id::UserId,
    },
    prelude::*,
};
use tokio::sync::Mutex;

use crate::rusther::EventSubHandler;

/// Arbitrates events to mutable event-(sub)-handlers.
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
/// This works by providing interior mutability via Arc<Mutex>.
///
/// Atomically-reference-counted (ARC) pointer clones for each sub-handler are distributed to
/// tasks per event invocation, each pointing to the mutex guarding the event handler.
pub struct Arbiter {
    command_prefix: char,
    commands: HashMap<&'static str, Arc<Mutex<dyn EventSubHandler>>>,
    user_id: Arc<Mutex<UserId>>,
}

impl Arbiter {
    pub fn new() -> Self {
        Self {
            command_prefix: '!',
            commands: HashMap::new(),
            user_id: Arc::new(Mutex::new(UserId::default())),
        }
    }
    // TODO: Make providing a name optional
    pub fn register_event_handler(
        &mut self,
        name: &'static str,
        handler: impl EventSubHandler + 'static,
    ) -> Result<(), String> {
        if self
            .commands
            .insert(name, Arc::new(Mutex::new(handler)))
            .is_some()
        {
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
    fn clone_handlers(&self) -> Vec<Arc<Mutex<dyn EventSubHandler>>> {
        self.commands.values().cloned().collect()
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

            let handlers = self.clone_handlers();

            for handler in handlers {
                let sub_ctx = safe_ctx.clone();
                let sub_msg = safe_msg.clone();

                tokio::spawn(async move {
                    let mut handler = handler.lock().await;
                    handler.message(sub_ctx, sub_msg).await;
                });
            }
        }
    }
    async fn message_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Message>,
        _new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        if let Some(user) = &event.author {
            if user.id == ctx.cache.current_user_id() {
                log::trace!("Skipping own message_update");
                return;
            }
        }
        let ctx = Arc::new(ctx);
        let event = Arc::new(event);

        let handlers = self.clone_handlers();

        for handler in handlers {
            let ctx = ctx.clone();
            let event = event.clone();

            tokio::spawn(async move {
                let mut handler = handler.lock().await;
                handler.message_update(ctx, event).await;
            });
        }
    }
    async fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let handlers = self.clone_handlers();

        tokio::spawn(async move {
            let ctx = Arc::new(ctx);
            let add_reaction = Arc::new(add_reaction);

            if let Some(user_id) = add_reaction.user_id {
                if user_id == ctx.cache.current_user_id() {
                    log::trace!("Skipping own reaction_add");
                    return;
                }
            }

            for handler in handlers {
                let ctx = ctx.clone();
                let add_reaction = add_reaction.clone();

                tokio::spawn(async move {
                    let mut handler = handler.lock().await;
                    handler.reaction_add(ctx, add_reaction).await;
                });
            }
        });
    }
    async fn ready(&self, ctx: Context, ready: Ready) {
        // This function should remain the simplest event forwarder, as example.

        // Clones of these Arc<> will be moved to child tasks.
        let ctx = Arc::new(ctx);
        let ready = Arc::new(ready);

        /* self.commands is a list of Arc<>, so cloning it is not cloning the handlers.
           Instead, it is cloning pointers to the handlers, which are locked behind mutexes.

           Access to the state mutex is kept minimal and it is released as soon as possible
           by dropping from scope.
        */
        let mut user_id = self.user_id.lock().await;
        *user_id = ready.user.id;
        drop(user_id);

        let handlers = self.clone_handlers();

        // Spawn a task for each sub-handler.
        for handler in handlers {
            let ctx = ctx.clone();
            let ready = ready.clone();

            tokio::spawn(async move {
                // Each task will wait for and then give input to a sub-handler.
                let mut handler = handler.lock().await;
                handler.ready(ctx, ready).await;
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

        let result = arbiter.register_event_handler("test", UnitRecipient);
        assert!(result.is_ok());
    }

    #[test]
    fn register_text_command_duplicate() {
        let mut arbiter = Arbiter::new();

        let result = arbiter.register_event_handler("test", UnitRecipient);
        assert!(result.is_ok());

        let result = arbiter.register_event_handler("test", UnitRecipient);
        assert!(result.is_err());
    }

    #[test]
    fn sanitize_simple_message() {
        let input = "!lorem ipsum".to_string();
        let actual = Arbiter::sanitize(input);
        assert_eq!("lorem ipsum", &actual);
    }
}
