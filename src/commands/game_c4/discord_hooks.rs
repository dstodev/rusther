use std::collections::HashMap;

use serenity::{
	async_trait,
	model::{
		channel::{
			Message,
			Reaction,
			ReactionType,
		},
		id::MessageId,
	},
	prelude::*,
};
use tokio::task::yield_now;

use crate::{
	commands::game_c4::c4::{
		ConnectFour,
		GameState,
		Player,
	},
	rusther::EventSubHandler,
};

struct ConnectFourContext {
	game: ConnectFour,
	message: Message,
	reactions: Vec<Reaction>,
}

impl ConnectFourContext {
	pub fn new(game: ConnectFour, message: Message) -> Self {
		Self {
			game,
			message,
			reactions: Vec::new(),
		}
	}
}

pub struct ConnectFourDiscord {
	games: HashMap<MessageId, ConnectFourContext>,
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
	async fn message(&mut self, ctx: &Context, new_message: &Message) {
		let message = &new_message.content;

		if message.as_str() == "c4 start" {
			let game = ConnectFour::new(7, 6);
			let say = Self::get_render_string(&game);

			if let Ok(message) = new_message.channel_id.say(&ctx.http, say).await {
				let id = message.id;
				let context = ConnectFourContext::new(game, message);

				if self.games.insert(id, context).is_some() {
					log::debug!("C4 hashmap key collision!");
				}
				if let Some(context) = self.games.get_mut(&id) {
					let reaction_cache = &mut context.reactions;
					let width = context.game.board.width();
					Self::add_reactions_to(&context.message, reaction_cache, width, ctx).await;
				}
			}
		} else if message.as_str() == "c4 purge" {
			let map = Vec::from_iter(self.games.drain());

			for (_id, context) in map {
				Self::finalize_game(context, ctx).await;
			}
		}
	}
	async fn reaction_add(&mut self, ctx: &Context, add_reaction: &Reaction) {
		let id = add_reaction.message_id;

		if let Some(context) = self.games.get_mut(&id) {
			let game = &mut context.game;
			let reaction_unicode = &add_reaction.emoji.as_data();

			let should_respond = game.state == GameState::Playing
				&& reaction_unicode.ends_with("\u{fe0f}\u{20e3}");

			if should_respond {
				let column = reaction_unicode.as_bytes()[0] - 0x30;

				if game.emplace(column.into()) {
					if game.state == GameState::Playing {
						let message = &mut context.message;
						Self::render_to_message(game, message, ctx).await;
					} else if let Some(context) = self.games.remove(&id) {  // The game has ended, remove it from memory.
						Self::finalize_game(context, ctx).await;
					}
				}
				if let Err(reason) = add_reaction.delete(&ctx.http).await {
					log::debug!("Could not remove reaction because {:?}", reason);
				};
			}
		}
	}
}

impl ConnectFourDiscord {
	async fn add_reactions_to(message: &Message, reaction_cache: &mut Vec<Reaction>, width: i32, ctx: &Context) {
		for column in 0..width {
			let reaction = Self::get_reaction_for_column(column);

			// Add one-at-a-time to ensure they are added in order
			match message.react(&ctx.http, reaction).await {
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
		// Using unicode keycap symbols of the form <ascii value for number><unicode fe0f 20e3>,
		// see: https://unicode.org/emoji/charts-12.0/full-emoji-list.html#0030_fe0f_20e3
		format!("{}\u{fe0f}\u{20e3}", column)
	}
	async fn finalize_game(mut context: ConnectFourContext, ctx: &Context) {
		// If a player has won, do not override the game state to closed i.e. 'draw'.
		if context.game.state == GameState::Playing {
			context.game.state = GameState::Closed;
		}
		Self::render_to_message(&context.game, &mut context.message, ctx).await;
		Self::delete_reactions(context.reactions, ctx).await;
	}
	async fn render_to_message(game: &ConnectFour, message: &mut Message, ctx: &Context) {
		let say = Self::get_render_string(game);

		if let Err(reason) = message.edit(&ctx.http, |builder| builder.content(say)).await {
			log::debug!("Could not edit message because {:?}", reason);
		}
	}
	fn get_render_string(game: &ConnectFour) -> String {
		format!("{}{}{}",
		        Self::get_header_string(game),
		        Self::get_board_string(game),
		        Self::get_axis_string(game),
		)
	}
	async fn delete_reactions(reactions: Vec<Reaction>, ctx: &Context) {
		let http = ctx.http.clone();

		tokio::spawn(async move {
			for reaction in reactions {
				let _ = reaction.delete(http.clone()).await;
				yield_now().await;
			}
		});
	}
	fn get_header_string(game: &ConnectFour) -> String {
		return if game.state == GameState::Playing {
			let player = Some(game.turn);
			format!("> Current turn: {}\n", Self::get_player_label(player))
		} else {
			let player = game.get_winner();
			format!("> {} player wins!\n", Self::get_player_label(player))
		};
	}
	fn get_board_string(game: &ConnectFour) -> String {
		let mut board = String::new();

		for row in 0..game.board.height() {
			for column in 0..game.board.width() {
				let player = game.board.get(row, column).cloned();
				board += Self::get_player_token(player);
				board += " ";
			}
			board += "\n";
		}
		board
	}
	fn get_player_label(player: Option<Player>) -> String {
		format!("{} {}", Self::get_player_token(player), match player {
			Some(Player::Red) => "Red",
			Some(Player::Blue) => "Blue",
			None => "No",
		})
	}
	fn get_player_token(player: Option<Player>) -> &'static str {
		match player {
			Some(Player::Red) => ":red_circle:",
			Some(Player::Blue) => ":blue_circle:",
			None => ":black_circle:",
		}
	}
	fn get_axis_string(game: &ConnectFour) -> String {
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
}
