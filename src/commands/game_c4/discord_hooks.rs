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

use crate::{
	commands::game_c4::c4::{
		ConnectFour,
		GameState,
		Player,
	},
	rusther::EventSubHandler,
};

pub struct ConnectFourDiscord {
	games: HashMap<MessageId, ConnectFour>,
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
			let mut game = ConnectFour::new(7, 6);

			game.restart();

			let say = Self::get_render_string(&game);

			if let Ok(message) = new_message.channel_id.say(&ctx.http, say).await {
				let id = message.id;
				let width = game.board.width();

				if self.games.insert(id, game).is_some() {
					panic!("C4 hashmap key collision!");
				}

				Self::add_reactions_to(&message, width, ctx).await;
			}
		};
	}
	async fn reaction_add(&mut self, ctx: &Context, add_reaction: &Reaction) {
		let id = add_reaction.message_id;

		if let Some(game) = self.games.get_mut(&id) {
			let reaction_unicode = &add_reaction.emoji.as_data();

			let should_respond = game.state == GameState::Playing
				&& reaction_unicode.ends_with("\u{fe0f}\u{20e3}");

			if should_respond {
				let column = reaction_unicode.as_bytes()[0] - 0x30;

				if game.emplace(column.into()) {
					let channel_id = add_reaction.channel_id;

					if let Ok(mut message) = channel_id.message(&ctx.http, id).await {
						let say = Self::get_render_string(game);

						if let Err(reason) = message.edit(&ctx.http, |builder| builder.content(say)).await {
							println!("Could not edit message because {:?}", reason);
						}
						if game.state == GameState::Closed || matches!(game.state, GameState::Won { .. }) {
							let width = game.board.width();
							Self::remove_reactions_from(&message, width, ctx).await;

							self.games.remove(&id);
						}
					}
				}
				if let Err(reason) = add_reaction.delete(&ctx.http).await {
					println!("Could not remove reaction because {:?}", reason);
				};
			}
		}
	}
}

impl ConnectFourDiscord {
	async fn add_reactions_to(message: &Message, width: i32, ctx: &Context) {
		for column in 0..width {
			let reaction = Self::get_reaction_for_column(column);

			// Add one-at-a-time to ensure they are added in order
			if let Err(reason) = message.react(&ctx.http, reaction).await {
				println!("Could not react because {:?}", reason);
			}
		}
	}
	async fn remove_reactions_from(message: &Message, width: i32, ctx: &Context) {
		for column in 0..width {
			let emoji = Self::get_reaction_for_column(column);

			if let Err(reason) = message.delete_reaction_emoji(&ctx.http, emoji).await {
				println!("Could not remove reaction because {:?}", reason);
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
	fn get_render_string(game: &ConnectFour) -> String {
		format!("{}{}{}",
		        Self::get_header_string(game),
		        Self::get_board_string(game),
		        Self::get_axis_string(game),
		)
	}
	fn get_header_string(game: &ConnectFour) -> String {
		let player_str = |p| match p {
			Some(Player::Red) => "Red",
			Some(Player::Blue) => "Blue",
			None => "No",
		};

		if game.state == GameState::Playing {
			format!("Current turn: {}\n", player_str(Some(game.turn)))
		} else {
			format!("{} player wins!\n", player_str(game.get_winner()))
		}
	}
	fn get_board_string(game: &ConnectFour) -> String {
		let mut board = String::new();

		let player_str = |p| match p {
			Some(Player::Red) => ":red_circle:",
			Some(Player::Blue) => ":blue_circle:",
			None => ":black_circle:",
		};

		for row in 0..game.board.height() {
			for column in 0..game.board.width() {
				let player = game.board.get(row, column).cloned();
				board += player_str(player);
				board += " ";
			}
			board += "\n";
		}
		board
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
