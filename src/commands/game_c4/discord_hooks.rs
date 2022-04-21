use std::collections::HashMap;
use std::sync::Arc;

use serenity::{
	async_trait,
	http::Http,
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

#[derive(Debug)]
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

impl ConnectFourContext {
	async fn render(&mut self, http: &Arc<Http>) {
		let say = self.get_render_string();
		let message = &mut self.message;

		if let Err(reason) = message.edit(http, |builder| builder.content(say)).await {
			log::debug!("Could not edit message because {:?}", reason);
		}
	}
	async fn delete_reactions(&mut self, http: &Arc<Http>) {
		for reaction in self.reactions.drain(..) {
			let _ = reaction.delete(http).await;
		}
	}
	fn get_render_string(&self) -> String {
		format!("{}{}{}",
		        self.get_header_string(),
		        self.get_board_string(),
		        self.get_axis_string(),
		)
	}
	fn get_header_string(&self) -> String {
		let game = &self.game;

		return if game.state == GameState::Playing {
			let player = Some(game.turn);
			format!("> Current turn: {}\n", Self::get_player_label(player))
		} else {
			let player = game.get_winner();
			format!("> {} player wins!\n", Self::get_player_label(player))
		};
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
				let player = game.board.get(row, column).cloned();
				board += Self::get_player_token(player);
				board += " ";
			}
			board += "\n";
		}
		board
	}
	async fn add_reactions(&mut self, http: &Arc<Http>) {
		let width = self.game.board.width();
		let message = &self.message;
		let reaction_cache = &mut self.reactions;

		for column in 0..width {
			let reaction = Self::get_reaction_for_column(column);

			// Add one-at-a-time to ensure they are added in order
			match message.react(http, reaction).await {
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
	async fn finalize(&mut self, http: &Arc<Http>) {
		// If a player has won, do not override the game state to closed i.e. 'draw'.
		if self.game.state == GameState::Playing {
			self.game.state = GameState::Closed;
		}
		self.render(http).await;
		self.delete_reactions(http).await;
	}
}

pub struct ConnectFourDiscord {
	games: HashMap<MessageId, Arc<Mutex<ConnectFourContext>>>,
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
	async fn message(&mut self, ctx: Arc<Context>, new_message: Arc<Message>) {
		let message = &new_message.content;

		match message.as_str() {
			"c4 start" => {
				let say = ":anchor:";

				match new_message.channel_id.say(&ctx.http, say).await {
					Ok(message) => {
						let id = message.id;
						let game = ConnectFour::new(7, 6);
						let context = ConnectFourContext::new(game, message);

						if self.games.insert(id, Arc::new(Mutex::new(context))).is_some() {
							log::debug!("Hashmap key collision!");
						}
						if let Some(mutex) = self.games.get(&id).cloned() {
							let mut context = mutex.lock().await;
							context.render(&ctx.http).await;
							context.add_reactions(&ctx.http).await;
						}
					}
					Err(reason) => log::debug!("Could not send anchor message because {:?}", reason),
				}
			}
			"c4 purge" => {
				for (_id, mutex) in self.games.drain() {
					let http = ctx.http.clone();

					tokio::spawn(async move {
						let mut instance = mutex.lock().await;
						instance.finalize(&http).await;
					});
				}
			}
			_ => {}
		}
	}
	async fn reaction_add(&mut self, ctx: Arc<Context>, add_reaction: Arc<Reaction>) {
		let id = add_reaction.message_id;
		let mut game_has_ended: bool = false;

		if let Some(mutex) = self.games.get(&id).cloned() {
			let reaction_unicode = add_reaction.emoji.as_data();
			let sub_http = ctx.http.clone();

			tokio::spawn(async move {
				if let Err(reason) = add_reaction.delete(&sub_http).await {
					log::debug!("Could not remove reaction because {:?}", reason);
				};
			});

			/* .lock_owned() should be used when the state pointer is to be moved to another task.
			     However, even though the state pointer is moved to a separate task, the state is
			     still locked, so sub-tasks should remain minimal.
		    */
			let mut instance = mutex.lock_owned().await;
			let game = &mut instance.game;

			let should_respond = game.state == GameState::Playing
				&& reaction_unicode.ends_with("\u{fe0f}\u{20e3}");

			if should_respond {
				let column = reaction_unicode.as_bytes()[0] - 0x30;

				if game.emplace(column.into()) && game.state != GameState::Playing {
					// Signal that the game has ended, ...
					game_has_ended = true;
				}
				let http = ctx.http.clone();
				tokio::spawn(async move {
					/* TODO: How can we drop intermediate render requests, given a user provides
					         many simultaneous reactions?
					 */
					instance.render(&http).await;
				});
			}
			// ... and release the instance lock.
		}
		if game_has_ended {
			/* TODO: Figure out when to remove games.
			         If we use self.games.remove() here, there is no guarantee other tasks have
			         all completed, which may want to use the instance context. */

			log::info!("Game {} concluded!", id);

			if let Some(mutex) = self.games.get(&id) {
				let mut instance = mutex.lock().await;
				instance.finalize(&ctx.http).await;
			}
		}
	}
}
