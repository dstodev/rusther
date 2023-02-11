use std::{collections::HashMap, sync::Arc};

use serenity::{
    async_trait,
    model::{
        channel::{Message, Reaction},
        id::MessageId,
    },
    prelude::*,
};
use tokio::sync::{Mutex, RwLock};

use crate::commands::game_c4::discord_message::InteractionMode;
use crate::rusther::EventSubHandler;

use super::{ConnectFour, ConnectFour1p, ConnectFour2p, DiscordMessage, GameStatus};

pub struct ConnectFourDiscord {
    games: Arc<RwLock<HashMap<MessageId, Arc<Mutex<DiscordMessage>>>>>,
}

impl ConnectFourDiscord {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventSubHandler for ConnectFourDiscord {
    async fn message(&mut self, context: Context, message: Message) {
        let games = self.games.clone();
        tokio::spawn(async move {
            let mut game_to_start: Option<Box<dyn ConnectFour + Send + Sync>> = None;
            let mut mode = InteractionMode::TwoPlayer;

            match message.content.as_str() {
                "c4 start" => game_to_start = Some(Box::new(ConnectFour2p::new(7, 6))),
                "c4 start random" | "c4 random" => {
                    mode = InteractionMode::OnePlayer;
                    game_to_start = Some(Box::new(ConnectFour1p::new(7, 6, None)));
                }
                "c4 purge" => {
                    let game_messages;
                    {
                        let mut games_write = games.write().await;
                        game_messages = Vec::from_iter(games_write.drain());
                    }
                    let http = context.http.clone();
                    for (_id, game) in game_messages {
                        let http = http.clone();

                        let mut game_lock = game.lock().await;
                        game_lock.finalize(http).await;
                    }
                }
                _ => {}
            }

            if let Some(game) = game_to_start {
                let say = ":anchor:";

                match message.channel_id.say(&context, say).await {
                    Ok(message) => {
                        let id = message.id;
                        let state = DiscordMessage::new(game, message, mode);
                        let state = Arc::new(Mutex::new(state));
                        {
                            let mut games_write = games.write().await;
                            if games_write.insert(id, state).is_some() {
                                log::debug!("Hashmap key collision!");
                            }
                        }
                        let game_arc;
                        {
                            let games_read = games.read().await;
                            game_arc = games_read.get(&id).cloned().unwrap();
                        }
                        let mut game_lock = game_arc.lock().await;
                        game_lock.render(&context).await;
                        game_lock.add_reactions(&context).await;
                    }
                    Err(reason) => {
                        log::debug!("Could not send anchor message because {:?}", reason)
                    }
                }
            }
        });
    }
    async fn reaction_add(&mut self, context: Context, reaction: Reaction) {
        let games = self.games.clone();
        tokio::spawn(async move {
            let id = reaction.message_id;
            let mut game_has_ended = false;

            let mut game_arc = None;
            {
                let games_read = games.read().await;
                if let Some(game) = games_read.get(&id).cloned() {
                    game_arc = Some(game);
                }
            }

            if let Some(game) = game_arc {
                let mut game_lock = game.lock().await;
                let reaction_unicode = reaction.emoji.as_data();

                let should_respond = game_lock.game.state() == GameStatus::Playing
                    && reaction_unicode.ends_with("\u{fe0f}\u{20e3}");

                if should_respond {
                    if let Err(reason) = reaction.delete(&context).await {
                        log::debug!("Could not remove reaction because {:?}", reason);
                    };

                    let column = reaction_unicode.as_bytes()[0] - 0x30;

                    if game_lock.game.emplace(column.into())
                        && game_lock.game.state() != GameStatus::Playing
                    {
                        game_has_ended = true;
                    }

                    if game_has_ended {
                        /* TODO: Figure out when to remove games.
                        If self.games.remove() here, there is no guarantee other tasks have
                        all completed, which may use the instance context. */

                        log::info!("Game {} has concluded!", id);
                        game_lock.finalize(context).await;
                    } else {
                        game_lock.render(context).await;
                    }
                }
            }
        });
    }
}
