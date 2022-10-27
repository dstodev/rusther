use std::collections::HashMap;

use serenity::{
    async_trait,
    http::CacheHttp,
    model::{
        channel::{Message, Reaction, ReactionType},
        id::MessageId,
    },
    prelude::*,
};

use crate::{
    commands::game_c4::c4::{ConnectFour, GameState, Player},
    log_scope_time,
    rusther::EventSubHandler,
};

#[derive(Debug)]
struct ConnectFourState {
    game: ConnectFour,
    message: Message,
    reactions: Vec<Reaction>,
}

impl ConnectFourState {
    pub fn new(game: ConnectFour, message: Message) -> Self {
        Self {
            game,
            message,
            reactions: Vec::new(),
        }
    }
}

impl ConnectFourState {
    async fn render(&mut self, http: impl CacheHttp) {
        log_scope_time!("Render");

        let say = self.get_render_string();

        if let Err(reason) = self
            .message
            .edit(http, |builder| builder.content(say))
            .await
        {
            log::debug!("Could not edit message because {:?}", reason);
        }
    }
    async fn delete_reactions(&mut self, http: impl CacheHttp) {
        log_scope_time!();
        for reaction in self.reactions.drain(..) {
            if let Err(reason) = reaction.delete_all(&http).await {
                log::debug!("Could not delete reactions because {:?}", reason);
            }
        }
    }
    fn get_render_string(&self) -> String {
        format!(
            "{}{}{}",
            self.get_header_string(),
            self.get_board_string(),
            self.get_axis_string(),
        )
    }
    fn get_header_string(&self) -> String {
        let game = &self.game;

        return if game.state == GameState::Playing {
            format!(
                "> Current turn: {}\n",
                Self::get_player_label(&Some(game.turn))
            )
        } else {
            format!(
                "> {} player wins!\n",
                Self::get_player_label(&game.get_winner())
            )
        };
    }
    fn get_player_label(player: &Option<Player>) -> String {
        format!(
            "{} {}",
            Self::get_player_token(player),
            match player {
                Some(Player::Red) => "Red",
                Some(Player::Blue) => "Blue",
                None => "No", // becomes e.g. "No player wins!"
            }
        )
    }
    fn get_player_token(player: &Option<Player>) -> &'static str {
        match player {
            Some(Player::Red) => ":red_circle:",
            Some(Player::Blue) => ":blue_circle:",
            None => ":black_circle:",
        }
    }
    fn get_axis_string(&self) -> String {
        let game = &self.game;
        let mut axis = String::new();

        if game.state == GameState::Playing {
            for column in 0..game.board.width() {
                axis += &Self::get_reaction_string_for_column(column);
                axis += " ";
            }
            axis += "\n";
        }
        axis
    }
    fn get_board_string(&self) -> String {
        let game = &self.game;
        let mut board = String::new();

        for row in 0..game.board.height() {
            for column in 0..game.board.width() {
                let player = match game.board.get(row, column) {
                    Some(v) => Some(v.value),
                    None => None,
                };
                board += Self::get_player_token(&player);
                board += " ";
            }
            board += "\n";
        }
        board
    }
    async fn add_reactions(&mut self, http: impl CacheHttp) {
        let width = self.game.board.width();
        let message = &self.message;
        let reaction_cache = &mut self.reactions;

        for column in 0..width {
            let reaction = Self::get_reaction_for_column(column);

            // Add one-at-a-time to ensure they are added in order
            match message.react(&http, reaction).await {
                Ok(reaction) => reaction_cache.push(reaction),
                Err(reason) => log::debug!("Could not react because {:?}", reason),
            }
        }
    }
    fn get_reaction_for_column(column: i32) -> ReactionType {
        assert!((0..10).contains(&column));
        let triplet = Self::get_reaction_string_for_column(column);
        ReactionType::Unicode(triplet)
    }
    fn get_reaction_string_for_column(column: i32) -> String {
        // Using unicode keycap symbols in the form <ascii value for number><unicode fe0f 20e3>,
        // see: https://unicode.org/emoji/charts-12.0/full-emoji-list.html#0030_fe0f_20e3
        format!("{}\u{fe0f}\u{20e3}", column)
    }
    async fn finalize(&mut self, http: impl CacheHttp) {
        // If a player has won, do not override the game state to closed i.e. 'draw'.
        if self.game.state == GameState::Playing {
            self.game.state = GameState::Closed;
        }
        self.render(&http).await;
        self.delete_reactions(&http).await;
    }
}

pub struct ConnectFourDiscord {
    games: HashMap<MessageId, ConnectFourState>,
}

impl ConnectFourDiscord {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }
}

#[async_trait]
impl EventSubHandler for ConnectFourDiscord {
    async fn message(&mut self, context: Context, message: Message) {
        match message.content.as_str() {
            "c4 start" => {
                let say = ":anchor:";

                match message.channel_id.say(&context, say).await {
                    Ok(message) => {
                        let id = message.id;
                        let game = ConnectFour::new(7, 6);
                        let state = ConnectFourState::new(game, message);

                        if self.games.insert(id, state).is_some() {
                            log::debug!("Hashmap key collision!");
                        }
                        if let Some(game) = self.games.get_mut(&id) {
                            game.render(&context).await;
                            game.add_reactions(&context).await;
                        }
                    }
                    Err(reason) => {
                        log::debug!("Could not send anchor message because {:?}", reason)
                    }
                }
            }
            "c4 purge" => {
                for (_id, mut game) in self.games.drain() {
                    let http = context.http.clone();
                    game.finalize(http).await;
                }
            }
            _ => {}
        }
    }
    async fn reaction_add(&mut self, context: Context, reaction: Reaction) {
        let id = reaction.message_id;
        let mut game_has_ended: bool = false;

        if let Some(state) = self.games.get_mut(&id) {
            let reaction_unicode = reaction.emoji.as_data();

            let should_respond = state.game.state == GameState::Playing
                && reaction_unicode.ends_with("\u{fe0f}\u{20e3}");

            if should_respond {
                if let Err(reason) = reaction.delete(&context.clone()).await {
                    log::debug!("Could not remove reaction because {:?}", reason);
                };

                let column = reaction_unicode.as_bytes()[0] - 0x30;

                if state.game.emplace(column.into()) && state.game.state != GameState::Playing {
                    game_has_ended = true;
                }

                if game_has_ended {
                    /* TODO: Figure out when to remove games.
                    If self.games.remove() here, there is no guarantee other tasks have
                    all completed, which may use the instance context. */

                    log::info!("Game {} has concluded!", id);
                    state.finalize(&context).await;
                } else {
                    state.render(&context).await;
                }
            }
        }
    }
}
