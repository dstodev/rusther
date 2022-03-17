use serenity::model::id::MessageId;

use super::board::{Board, Token, Direction};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
	Red,
	Blue,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState {
	Closed,
	Playing,
}

pub struct ConnectFour {
	state: GameState,
	turn: Player,
	board: Board<Player>,
	message_id: MessageId,
}

impl ConnectFour {
	pub fn new(width: i32, height: i32) -> Self {
		Self {
			state: GameState::Closed,
			turn: Player::Red,
			board: Board::new(width, height),
			message_id: MessageId::default(),
		}
	}
	pub fn dispatch(&mut self, message: &str) {
		match message {
			"c4 start" | "c4 restart" => self.restart(),
			_ => {}
		};
	}
	pub fn restart(&mut self) {
		self.state = GameState::Playing;
		self.turn = Player::Red;
		self.board = Board::new(self.board.get_width(), self.board.get_height());
		self.message_id = MessageId::default();
	}
	pub fn emplace(&mut self, column: i32) -> bool {
		if column >= self.board.get_width() {
			return false;
		}

		for row in (0..self.board.get_height()).rev() {
			if self.board.get(row, column).is_none() {
				self.board.set(row, column, self.turn);

				self.turn = match self.turn {
					Player::Red => Player::Blue,
					Player::Blue => Player::Red,
				};
				return true;
			}
		}
		false
	}
	pub fn get_winner(&self, row: i32, column: i32) -> Option<Player> {
		// @formatter:off
		let up_down = self.board.get_count_in_direction(row, column, Direction::North)
		            + self.board.get_count_in_direction(row, column, Direction::South);

		let left_right = self.board.get_count_in_direction(row, column, Direction::East)
		               + self.board.get_count_in_direction(row, column, Direction::West);

		let tl_br = self.board.get_count_in_direction(row, column, Direction::NorthWest)
		          + self.board.get_count_in_direction(row, column, Direction::SouthEast);

		let bl_tr = self.board.get_count_in_direction(row, column, Direction::SouthWest)
		          + self.board.get_count_in_direction(row, column, Direction::NorthEast);
		// @formatter:on

		println!("{}\n{}\n{}\n{}", up_down, left_right, tl_br, bl_tr);

		let max = up_down.max(left_right).max(tl_br).max(bl_tr);

		if max == 4 {
			Some(self.turn)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_default() {
		let cf = ConnectFour::new(7, 6);
		assert_eq!(GameState::Closed, cf.state);
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_restart() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert_eq!(GameState::Playing, cf.state);
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_dispatch_start() {
		let mut cf = ConnectFour::new(7, 6);
		cf.dispatch("c4 start");
		assert_eq!(GameState::Playing, cf.state);
	}

	#[test]
	fn test_dispatch_restart() {
		let mut cf = ConnectFour::new(7, 6);
		cf.dispatch("c4 restart");
		assert_eq!(GameState::Playing, cf.state);
	}

	#[test]
	fn test_emplace_col0() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert!(cf.emplace(0) /* column 0 */);

		/* In a default [7 wide] by [6 high] board, emplace(0) would place in the far left column:

			   0 1 2 3 4 5 6
			0  - - - - - - -   (0,0) refers to the top left, so this means a RED piece should be
			1  - - - - - - -   located at coordinate (5[row],0[col]).
			2  - - - - - - -   In a flat array, this is found with ([row * stride] + col).
			3  - - - - - - -
			4  - - - - - - -   After placing a marker, the turn should switch to BLUE.
			5  R - - - - - -
		*/
		assert_eq!(Some(&Token::new(5, 0, Player::Red)), cf.board.get(5, 0));
		assert_eq!(Player::Blue, cf.turn);
	}

	#[test]
	fn test_emplace_col0_twice() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert!(cf.emplace(0));
		assert!(cf.emplace(0));
		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -
			1  - - - - - - -
			2  - - - - - - -
			3  - - - - - - -
			4  B - - - - - -
			5  R - - - - - -
		*/
		assert_eq!(Some(&Token::new(5, 0, Player::Red)), cf.board.get(5, 0));
		assert_eq!(Some(&Token::new(4, 0, Player::Blue)), cf.board.get(4, 0));
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_emplace_col6() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert!(cf.emplace(6));
		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -
			1  - - - - - - -
			2  - - - - - - -
			3  - - - - - - -
			4  - - - - - - -
			5  - - - - - - R
		*/
		assert_eq!(Some(&Token::new(5, 6, Player::Red)), cf.board.get(5, 6));
		assert_eq!(Player::Blue, cf.turn);
	}

	#[test]
	fn test_emplace_col7_out_of_bounds() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert!(/* returns false */ !cf.emplace(7));
	}

	#[test]
	fn test_emplace_col0_six_times() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		for _ in 0..6 {
			assert!(cf.emplace(0));
		}
		/*
			   0 1 2 3 4 5 6
			0  B - - - - - -
			1  R - - - - - -
			2  B - - - - - -
			3  R - - - - - -
			4  B - - - - - -
			5  R - - - - - -
		*/
		assert_eq!(Some(&Token::new(5, 0, Player::Red)), cf.board.get(5, 0));
		assert_eq!(Some(&Token::new(4, 0, Player::Blue)), cf.board.get(4, 0));
		assert_eq!(Some(&Token::new(3, 0, Player::Red)), cf.board.get(3, 0));
		assert_eq!(Some(&Token::new(2, 0, Player::Blue)), cf.board.get(2, 0));
		assert_eq!(Some(&Token::new(1, 0, Player::Red)), cf.board.get(1, 0));
		assert_eq!(Some(&Token::new(0, 0, Player::Blue)), cf.board.get(0, 0));
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_emplace_col0_seven_times() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		for _ in 0..6 {
			assert!(cf.emplace(0));
		}
		assert!(/* returns false */ !cf.emplace(0));
		/*
			   0 1 2 3 4 5 6
			0  B - - - - - -
			1  R - - - - - -
			2  B - - - - - -
			3  R - - - - - -
			4  B - - - - - -
			5  R - - - - - -
		*/
		assert_eq!(Some(&Token::new(5, 0, Player::Red)), cf.board.get(5, 0));
		assert_eq!(Some(&Token::new(4, 0, Player::Blue)), cf.board.get(4, 0));
		assert_eq!(Some(&Token::new(3, 0, Player::Red)), cf.board.get(3, 0));
		assert_eq!(Some(&Token::new(2, 0, Player::Blue)), cf.board.get(2, 0));
		assert_eq!(Some(&Token::new(1, 0, Player::Red)), cf.board.get(1, 0));
		assert_eq!(Some(&Token::new(0, 0, Player::Blue)), cf.board.get(0, 0));
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_get_winner_none() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert_eq!(None, cf.get_winner(0, 0));
	}

	#[test]
	fn test_get_winner_4tall_mixed() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		for _ in 0..4 {
			assert!(cf.emplace(0));
		}
		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -
			1  - - - - - - -
			2  B - - - - - -
			3  R - - - - - -
			4  B - - - - - -
			5  R - - - - - -
		*/
		assert_eq!(None, cf.get_winner(5, 0));
	}

	#[test]
	fn test_get_winner_3tall_red() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		assert!(cf.emplace(0));  // R (5,0)
		assert!(cf.emplace(1));  // B (5,1)
		assert!(cf.emplace(0));  // R (4,0)
		assert!(cf.emplace(1));  // B (4,1)
		assert!(cf.emplace(0));  // R (3,0)

		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -
			1  - - - - - - -
			2  - - - - - - -
			3  R - - - - - -
			4  R B - - - - -
			5  R B - - - - -
		*/
		assert_eq!(None, cf.get_winner(5, 0));
	}

	#[test]
	fn test_get_winner_4tall_red() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		assert!(cf.emplace(0));  // R (5,0)
		assert!(cf.emplace(1));  // B (5,1)
		assert!(cf.emplace(0));  // R (4,0)
		assert!(cf.emplace(1));  // B (4,1)
		assert!(cf.emplace(0));  // R (3,0)
		assert!(cf.emplace(1));  // B (3,1)
		assert!(cf.emplace(0));  // R (2,0) victory

		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -   Red should win here.
			1  - - - - - - -
			2  R - - - - - -
			3  R B - - - - -
			4  R B - - - - -
			5  R B - - - - -
		*/
		assert_eq!(Some(Player::Red), cf.get_winner(5, 0));
	}

	// TODO: Test for greater-than-four connection

	#[cfg(disable)]
	#[test]
	fn test_get_count_in_direction() {
		let stride = 5;
		let mut cf = ConnectFour::new(Some(stride), Some(stride));
		cf.restart();

		cf.board = vec![None; stride * stride];
		cf.set_player_at_rc(2, 1, Player::Red);
		cf.set_player_at_rc(3, 1, Player::Red);
		cf.set_player_at_rc(0, 2, Player::Red);
		cf.set_player_at_rc(1, 2, Player::Red);
		cf.set_player_at_rc(2, 2, Player::Red);
		cf.set_player_at_rc(3, 2, Player::Red);
		cf.set_player_at_rc(1, 3, Player::Red);
		cf.set_player_at_rc(2, 3, Player::Blue);
		cf.set_player_at_rc(3, 3, Player::Blue);
		cf.set_player_at_rc(0, 4, Player::Red);
		cf.set_player_at_rc(2, 4, Player::Red);

		/*
			   0 1 2 3 4
			0  - - R - R
			1  - - R R -
			2  - R R B R  <-- Note single BLUE piece on this line at (2,3)
			3  - R R B -  <-- and here at (3,3)
			4  - - - - -
		*/
		let index_middle = index_from_rc(2, 2, stride);
		assert_eq!(3, cf.get_count_in_direction(index_middle, Direction::North));
		assert_eq!(3, cf.get_count_in_direction(index_middle, Direction::NorthEast));
		assert_eq!(1, cf.get_count_in_direction(index_middle, Direction::East));
		assert_eq!(1, cf.get_count_in_direction(index_middle, Direction::SouthEast));
		assert_eq!(2, cf.get_count_in_direction(index_middle, Direction::South));
		assert_eq!(2, cf.get_count_in_direction(index_middle, Direction::SouthWest));
		assert_eq!(2, cf.get_count_in_direction(index_middle, Direction::West));
		assert_eq!(1, cf.get_count_in_direction(index_middle, Direction::NorthWest));
	}
}
