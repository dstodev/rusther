use serenity::{
	async_trait,
	model::{
		channel::{
			Message,
			Reaction,
			ReactionType,
		},
	},
	prelude::*,
};

use crate::{
	commands::{
		ConnectFour,
		game_c4::c4::GameState,
	},
	rusther::EventSubHandler,
};

use super::c4::Player;

#[async_trait]
impl EventSubHandler for ConnectFour {
	async fn message(&mut self, ctx: &Context, new_message: &Message) {
		let message = &new_message.content;

		if message.as_str() == "c4 start" {
			self.restart();

			let say = self.get_render_string();

			if let Ok(m) = new_message.channel_id.say(&ctx.http, say).await {
				self.message_id = m.id;

				for column in 0..self.board.get_width() {
					let reaction = Self::get_reaction_for_column(column);

					// Add one-at-a-time to ensure they are added in order
					if let Err(reason) = m.react(&ctx.http, reaction).await {
						println!("Could not react because {:?}", reason);
					}
				}
			}
		};
	}

	async fn reaction_add(&mut self, ctx: &Context, add_reaction: &Reaction) {
		let should_react = self.state == GameState::Playing
			&& add_reaction.message_id == self.message_id;

		if should_react {
			let reaction_unicode = &add_reaction.emoji.as_data();

			if reaction_unicode.ends_with("\u{fe0f}\u{20e3}") {
				let column = reaction_unicode.as_bytes()[0] - 0x30;

				if self.emplace(column.into()) {
					let channel_id = add_reaction.channel_id;

					if let Ok(mut message) = channel_id.message(&ctx.http, self.message_id).await {
						let say = self.get_render_string();

						if let Err(reason) = message.edit(&ctx.http, |builder| builder.content(say)).await {
							println!("Could not edit message because {:?}", reason);
						}
						if self.get_winner().is_some() {
							for column in 0..self.board.get_width() {
								let emoji = Self::get_reaction_for_column(column);

								if let Err(reason) = message.delete_reaction_emoji(&ctx.http, emoji).await {
									println!("Could not remove reaction because {:?}", reason);
								}
							}
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

impl ConnectFour {
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
	fn get_render_string(&self) -> String {
		format!("{}{}{}",
		        self.get_header_string(),
		        self.get_board_string(),
		        self.get_axis_string(),
		)
	}
	fn get_header_string(&self) -> String {
		let player_str = |p| match p {
			Player::Red => "Red",
			Player::Blue => "Blue",
		};

		return if let Some(winner) = self.get_winner() {
			format!("{} player wins!\n", player_str(winner))
		} else {
			format!("Current turn: {}\n", player_str(self.turn))
		};
	}
	fn get_board_string(&self) -> String {
		let mut board = String::new();

		for row in 0..self.board.get_height() {
			for column in 0..self.board.get_width() {
				let player = self.board.get(row, column).cloned();
				board += Self::get_string_for(player);
			}
			board += "\n";
		}
		board
	}
	fn get_axis_string(&self) -> String {
		let mut axis = String::new();

		if self.state == GameState::Playing {
			for column in 0..self.board.get_width() {
				axis += &Self::get_reaction_string_for_column(column);
			}
			axis += "\n";
		}
		axis
	}
	fn get_string_for(player: Option<Player>) -> &'static str {
		match player {
			Some(Player::Red) => ":red_circle:",
			Some(Player::Blue) => ":blue_circle:",
			None => ":green_circle:",
		}
	}
}
