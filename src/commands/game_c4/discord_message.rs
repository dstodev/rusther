use serenity::{
    http::CacheHttp,
    model::channel::{Message, Reaction, ReactionType},
};

use crate::commands::game_c4::discord_message::InteractionMode::{OnePlayer, TwoPlayer};
use crate::log_scope_time;

use super::{ConnectFour, GameStatus, Player};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionMode {
    OnePlayer,
    TwoPlayer,
}

pub struct DiscordMessage {
    pub game: Box<dyn ConnectFour + Send + Sync>,
    message: Message,
    mode: InteractionMode,
    reactions: Vec<Reaction>,
}

impl DiscordMessage {
    pub fn new(
        game: Box<dyn ConnectFour + Send + Sync + 'static>,
        message: Message,
        mode: InteractionMode,
    ) -> Self {
        Self {
            game,
            message,
            mode,
            reactions: Vec::new(),
        }
    }
}

impl DiscordMessage {
    pub async fn render(&mut self, http: impl CacheHttp) {
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

        return if game.state() == GameStatus::Playing {
            format!(
                "> Current turn: {}\n",
                self.get_player_label(&Some(*game.turn()))
            )
        } else {
            format!("> {} wins!\n", self.get_player_label(&game.get_winner()))
        };
    }
    fn get_player_label(&self, player: &Option<Player>) -> String {
        format!(
            "{} {}",
            self.get_player_token(player),
            match player {
                Some(Player::Red) => match self.mode {
                    TwoPlayer => "Red",
                    OnePlayer => "Player",
                },
                Some(Player::Blue) => match self.mode {
                    TwoPlayer => "Blue",
                    OnePlayer => "Bot",
                },
                None => "Nobody", // becomes e.g. "Nobody wins!"
            }
        )
    }
    fn get_player_token(&self, player: &Option<Player>) -> &'static str {
        match player {
            Some(Player::Red) => match self.mode {
                TwoPlayer => ":red_circle:",
                OnePlayer => ":orange_circle:",
            },
            Some(Player::Blue) => match self.mode {
                TwoPlayer => ":blue_circle:",
                OnePlayer => ":purple_circle:",
            },
            None => ":black_circle:",
        }
    }
    fn get_axis_string(&self) -> String {
        let game = &self.game;
        let mut axis = String::new();

        if game.state() == GameStatus::Playing {
            for column in 0..game.board().width() {
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

        for row in 0..game.board().height() {
            for column in 0..game.board().width() {
                let player = match game.board().get(row, column) {
                    Some(v) => Some(v.value),
                    None => None,
                };
                board += self.get_player_token(&player);
                board += " ";
            }
            board += "\n";
        }
        board
    }
    pub async fn add_reactions(&mut self, http: impl CacheHttp) {
        let width = self.game.board().width();
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
    pub async fn finalize(&mut self, http: impl CacheHttp) {
        // If a player has won, do not override the game state to closed i.e. 'draw'.
        if self.game.state() == GameStatus::Playing {
            self.game.close();
        }
        self.render(&http).await;
        let _ = self.message.delete_reactions(&http).await;
    }
}
