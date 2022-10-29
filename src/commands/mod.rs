pub use game_c4::ConnectFourDiscord;
pub use message_ping::Ping;
pub use ready_announce::Announce;

mod game_c4;
mod message_ping;
mod ready_announce;

impl super::Arbiter {
	pub fn with_all_commands(mut self) -> Self {
		self.register_event_handler(Ping::new()).unwrap();
		self.register_event_handler(Announce).unwrap();
		self.register_event_handler(ConnectFourDiscord::new()).unwrap();
		self
	}
}
