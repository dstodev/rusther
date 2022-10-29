use std::collections::HashMap;

use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction},
        id::MessageId,
    },
    prelude::*,
};

use crate::rusther::EventSubHandler;

use super::{ConnectFour, ConnectFourMessage, GameStatus};

pub struct ConnectFourDiscord {
    game_messages: HashMap<MessageId, ConnectFourMessage>,
}

impl ConnectFourDiscord {
    pub fn new() -> Self {
        Self {
            game_messages: HashMap::new(),
        }
    }
}

#[async_trait]
impl EventSubHandler for ConnectFourDiscord {
    async fn message(&mut self, context: Context, message: Message) {
        //tokio::spawn(async move {
        match message.content.as_str() {
            "c4 start" => {
                let say = ":anchor:";

                match message.channel_id.say(&context, say).await {
                    Ok(message) => {
                        let id = message.id;
                        let game = ConnectFour::new(7, 6);
                        let state = ConnectFourMessage::new(game, message);

                        if self.game_messages.insert(id, state).is_some() {
                            log::debug!("Hashmap key collision!");
                        }
                        if let Some(message) = self.game_messages.get_mut(&id) {
                            message.render(&context).await;
                            message.add_reactions(&context).await;
                        }
                    }
                    Err(reason) => {
                        log::debug!("Could not send anchor message because {:?}", reason)
                    }
                }
            }
            "c4 purge" => {
                for (_id, mut game) in self.game_messages.drain() {
                    let http = context.http.clone();
                    game.finalize(http).await;
                }
            }
            _ => {}
        }
        //});
    }
    async fn reaction_add(&mut self, context: Context, reaction: Reaction) {
        //tokio::spawn(async move {
        let id = reaction.message_id;
        let mut game_has_ended = false;

        if let Some(message) = self.game_messages.get_mut(&id) {
            let reaction_unicode = reaction.emoji.as_data();

            let should_respond = message.game.state == GameStatus::Playing
                && reaction_unicode.ends_with("\u{fe0f}\u{20e3}");

            if should_respond {
                if let Err(reason) = reaction.delete(&context.clone()).await {
                    log::debug!("Could not remove reaction because {:?}", reason);
                };

                let column = reaction_unicode.as_bytes()[0] - 0x30;

                if message.game.emplace(column.into()) && message.game.state != GameStatus::Playing
                {
                    game_has_ended = true;
                }

                if game_has_ended {
                    /* TODO: Figure out when to remove games.
                    If self.games.remove() here, there is no guarantee other tasks have
                    all completed, which may use the instance context. */

                    log::info!("Game {} has concluded!", id);
                    message.finalize(&context).await;
                } else {
                    message.render(&context).await;
                }
            }
        }
        //});
    }
}
