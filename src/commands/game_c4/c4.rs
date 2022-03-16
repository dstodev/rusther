use serenity::model::id::MessageId;

use crate::commands::game_c4::{
	get_neighbor::get_neighbor,
	index_from_rc::index_from_rc,
};

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
	North,
	NorthEast,
	East,
	SouthEast,
	South,
	SouthWest,
	West,
	NorthWest,
}

const DEFAULT_BOARD_WIDTH: usize = 7;
const DEFAULT_BOARD_HEIGHT: usize = 6;

pub struct ConnectFour {
	state: GameState,
	turn: Player,
	board: Vec<Option<Player>>,
	board_width: usize,
	board_height: usize,
	current_index: usize,
	message_id: MessageId,
}

impl ConnectFour {
	pub fn new(width: Option<usize>, height: Option<usize>) -> Self {
		let board_width = width.unwrap_or(DEFAULT_BOARD_WIDTH);
		let board_height = height.unwrap_or(DEFAULT_BOARD_HEIGHT);

		Self {
			state: GameState::Closed,
			turn: Player::Red,
			board: vec![None; board_width * board_height],
			board_width,
			board_height,
			current_index: 0,
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
		self.board = vec![None; self.board_width * self.board_height];
		self.message_id = MessageId::default();
	}
	pub fn emplace(&mut self, column: usize) -> bool {
		if column >= self.board_width {
			return false;
		}

		for row in (0..self.board_height).rev() {
			if self.get_player_at_rc(row, column).is_none() {
				self.set_player_at_rc(row, column, self.turn);

				self.turn = match self.turn {
					Player::Red => Player::Blue,
					Player::Blue => Player::Red,
				};
				return true;
			}
		}
		false
	}
	pub fn get_winner(&self) -> Option<Player> {
		// @formatter:off
		let up_down = self.get_count_in_direction(self.current_index, Direction::North)
		            + self.get_count_in_direction(self.current_index, Direction::South);

		let left_right = self.get_count_in_direction(self.current_index, Direction::East)
		               + self.get_count_in_direction(self.current_index, Direction::West);

		let tl_br = self.get_count_in_direction(self.current_index, Direction::NorthWest)
		          + self.get_count_in_direction(self.current_index, Direction::SouthEast);

		let bl_tr = self.get_count_in_direction(self.current_index, Direction::SouthWest)
		          + self.get_count_in_direction(self.current_index, Direction::NorthEast);
		// @formatter:on

		println!("{}\n{}\n{}\n{}", up_down, left_right, tl_br, bl_tr);

		let max = up_down.max(left_right).max(tl_br).max(bl_tr);

		if max == 4 {
			Some(self.turn)
		} else {
			None
		}
	}
	fn set_player_at_rc(&mut self, row: usize, column: usize, player: Player) {
		let index = index_from_rc(row, column, self.board_width);
		self.board[index] = Some(player);
	}
	fn get_player_at_rc(&mut self, row: usize, column: usize) -> Option<Player> {
		let index = index_from_rc(row, column, self.board_width);
		self.board[index]
	}
	fn get_count_in_direction(&self, index: usize, direction: Direction) -> usize {
		if let Some(player) = self.board[index] {
			if let Some(neighbor) = get_neighbor(index, direction, self.board_width, self.board_height) {
				if let Some(other_piece) = self.board[neighbor] {
					if player == other_piece {
						return 1 + self.get_count_in_direction(neighbor, direction);
					}
				}
			}
			1
		} else {
			0
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_default() {
		let cf = ConnectFour::new(None, None);
		assert_eq!(GameState::Closed, cf.state);
		assert_eq!(Player::Red, cf.turn);
		assert_eq!(7, cf.board_width);
		assert_eq!(6, cf.board_height);
		assert_eq!(7 * 6, cf.board.len());
		assert!(cf.board.iter().all(|item| *item == None));
	}

	#[test]
	fn test_new_nondefault() {
		let cf = ConnectFour::new(Some(2), Some(4));
		assert_eq!(GameState::Closed, cf.state);
		assert_eq!(Player::Red, cf.turn);
		assert_eq!(2, cf.board_width);
		assert_eq!(4, cf.board_height);
		assert_eq!(2 * 4, cf.board.len());
		assert!(cf.board.iter().all(|item| *item == None));
	}

	#[test]
	fn test_restart() {
		let mut cf = ConnectFour::new(None, None);
		cf.restart();
		assert_eq!(GameState::Playing, cf.state);
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_dispatch_start() {
		let mut cf = ConnectFour::new(None, None);
		cf.dispatch("c4 start");
		assert_eq!(GameState::Playing, cf.state);
	}

	#[test]
	fn test_dispatch_restart() {
		let mut cf = ConnectFour::new(None, None);
		cf.dispatch("c4 restart");
		assert_eq!(GameState::Playing, cf.state);
	}

	#[test]
	fn test_emplace_col0() {
		let mut cf = ConnectFour::new(None, None);
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
		let row = 5;
		let col = 0;
		let stride = cf.board_width;
		assert_eq!(Some(Player::Red), cf.board[row * stride + col]);
		assert_eq!(Player::Blue, cf.turn);
	}

	#[test]
	fn test_emplace_col0_twice() {
		let mut cf = ConnectFour::new(None, None);
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

		let col = 0;
		let stride = cf.board_width;
		assert_eq!(Some(Player::Red), cf.board[5 * stride + col]);
		assert_eq!(Some(Player::Blue), cf.board[4 * stride + col]);
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_emplace_col6() {
		let mut cf = ConnectFour::new(None, None);
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
		let row = 5;
		let col = 6;
		let stride = cf.board_width;
		assert_eq!(Some(Player::Red), cf.board[row * stride + col]);
		assert_eq!(Player::Blue, cf.turn);
	}

	#[test]
	fn test_emplace_col7_out_of_bounds() {
		let mut cf = ConnectFour::new(None, None);
		cf.restart();
		assert!(/* returns false */ !cf.emplace(7));

		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -
			1  - - - - - - -
			2  - - - - - - -
			3  - - - - - - -
			4  - - - - - - -
			5  - - - - - - -
		*/
		assert!(cf.board.iter().all(|item| *item == None));
	}

	#[test]
	fn test_emplace_col0_six_times() {
		let mut cf = ConnectFour::new(None, None);
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

		let col = 0;
		let stride = cf.board_width;
		assert_eq!(Some(Player::Red), cf.board[5 * stride + col]);
		assert_eq!(Some(Player::Blue), cf.board[4 * stride + col]);
		assert_eq!(Some(Player::Red), cf.board[3 * stride + col]);
		assert_eq!(Some(Player::Blue), cf.board[2 * stride + col]);
		assert_eq!(Some(Player::Red), cf.board[1 * stride + col]);
		assert_eq!(Some(Player::Blue), cf.board[0 * stride + col]);
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_emplace_col0_seven_times() {
		let mut cf = ConnectFour::new(None, None);
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

		let col = 0;
		let stride = cf.board_width;
		assert_eq!(Some(Player::Red), cf.board[5 * stride + col]);
		assert_eq!(Some(Player::Blue), cf.board[4 * stride + col]);
		assert_eq!(Some(Player::Red), cf.board[3 * stride + col]);
		assert_eq!(Some(Player::Blue), cf.board[2 * stride + col]);
		assert_eq!(Some(Player::Red), cf.board[1 * stride + col]);
		assert_eq!(Some(Player::Blue), cf.board[0 * stride + col]);
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_get_winner_none() {
		let mut cf = ConnectFour::new(None, None);
		cf.restart();
		assert_eq!(None, cf.get_winner());
	}

	#[test]
	fn test_get_winner_4tall_mixed() {
		let mut cf = ConnectFour::new(None, None);
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
		assert_eq!(None, cf.get_winner());
	}

	#[test]
	fn test_get_winner_3tall_red() {
		let mut cf = ConnectFour::new(None, None);
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
		assert_eq!(None, cf.get_winner());
	}

	#[test]
	fn test_get_winner_4tall_red() {
		let mut cf = ConnectFour::new(None, None);
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
		assert_eq!(Some(Player::Red), cf.get_winner());
	}

	// TODO: Test for greater-than-four connection

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
