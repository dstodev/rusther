use std::fmt::{Display, Formatter};
use std::ops::Not;

use super::board::{Board, Direction};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
	Red,
	Blue,
}

impl Display for Player {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let say = match self {
			Self::Red => 'R',
			Self::Blue => 'B',
		};
		write!(f, "{}", say)
	}
}

impl Not for Player {
	type Output = Player;
	fn not(self) -> Self::Output {
		match self {
			Player::Red => Player::Blue,
			Player::Blue => Player::Red,
		}
	}
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameState {
	Closed,
	Playing,
	Won { player: Player },
}

pub struct ConnectFour {
	pub turn: Player,
	pub state: GameState,
	pub board: Board<Player>,
	last_pos_r: i32,
	last_pos_c: i32,
}

impl ConnectFour {
	pub fn new(width: i32, height: i32) -> Self {
		Self {
			state: GameState::Closed,
			turn: Player::Red,
			board: Board::new(width, height),
			last_pos_r: 0,
			last_pos_c: 0,
		}
	}
	pub fn restart(&mut self) {
		self.state = GameState::Playing;
		self.turn = Player::Red;
		self.board = Board::new(self.board.width(), self.board.height());
	}
	pub fn emplace(&mut self, column: i32) -> bool {
		let valid_move = self.state == GameState::Playing
			&& 0 <= column
			&& column < self.board.width();

		if valid_move {
			for row in (0..self.board.height()).rev() {
				if self.board.get(row, column).is_none() {
					self.board.set(row, column, self.turn);
					self.last_pos_r = row;
					self.last_pos_c = column;

					if let Some(player) = self.get_winner() {
						//self.board.fill(winner);  // Cool effect, but obscures the winning move
						self.state = GameState::Won { player };
					} else if self.board.data().iter().all(|e| e.is_some()) {
						// Board is full, but there are no winners. A draw!
						self.state = GameState::Closed;
					}
					self.turn = !self.turn;
					return true;
				}
			}
		}
		false
	}
	pub fn get_winner(&self) -> Option<Player> {
		if let GameState::Won { player } = self.state {
			return Some(player);
		}

		let row = self.last_pos_r;
		let column = self.last_pos_c;

		// @formatter:off
		let up_down = self.get_count_in_direction(row, column, Direction::North)
		            + self.get_count_in_direction(row, column, Direction::South) - 1;

		let left_right = self.get_count_in_direction(row, column, Direction::East)
		               + self.get_count_in_direction(row, column, Direction::West) - 1;

		let tl_br = self.get_count_in_direction(row, column, Direction::NorthWest)
		          + self.get_count_in_direction(row, column, Direction::SouthEast) - 1;

		let bl_tr = self.get_count_in_direction(row, column, Direction::SouthWest)
		          + self.get_count_in_direction(row, column, Direction::NorthEast) - 1;
		// @formatter:on

		let max = up_down.max(left_right).max(tl_br).max(bl_tr);

		if max >= 4 {
			Some(self.turn)
		} else {
			None
		}
	}
	fn get_count_in_direction(&self, row: i32, column: i32, direction: Direction) -> i32 {
		if let Some(lhs) = self.board.get(row, column) {
			if let Some(rhs) = self.board.get_neighbor(row, column, direction) {
				if lhs == rhs.value {
					return 1 + self.get_count_in_direction(rhs.row, rhs.column, direction);
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
		assert_eq!(Some(&Player::Red), cf.board.get(5, 0));
		assert_eq!(Player::Blue, cf.turn);
	}

	#[test]
	fn test_emplace_col0_when_closed() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		cf.state = GameState::Closed;
		assert!(/* returns false */ !cf.emplace(0));
		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -
			1  - - - - - - -
			2  - - - - - - -
			3  - - - - - - -
			4  - - - - - - -
			5  - - - - - - -
		*/
		assert_eq!(None, cf.board.get(5, 0));
		assert_eq!(Player::Red, cf.turn);
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
		assert_eq!(Some(&Player::Red), cf.board.get(5, 0));
		assert_eq!(Some(&Player::Blue), cf.board.get(4, 0));
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
		assert_eq!(Some(&Player::Red), cf.board.get(5, 6));
		assert_eq!(Player::Blue, cf.turn);
	}

	#[test]
	fn test_emplace_col7_out_of_bounds() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert_eq!(Player::Red, cf.turn);
		assert!(/* returns false */ !cf.emplace(7));
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_emplace_coln1_out_of_bounds() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert_eq!(Player::Red, cf.turn);
		assert!(/* returns false */ !cf.emplace(-1));
		assert_eq!(Player::Red, cf.turn);
	}

	#[test]
	fn test_emplace_col0_six_times() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		for _ in 0..6 {
			assert!(cf.emplace(0));
		}
		assert_eq!(Player::Red, cf.turn);
		/*
			   0 1 2 3 4 5 6
			0  B - - - - - -
			1  R - - - - - -
			2  B - - - - - -
			3  R - - - - - -
			4  B - - - - - -
			5  R - - - - - -
		*/
		assert_eq!(Some(&Player::Red), cf.board.get(5, 0));
		assert_eq!(Some(&Player::Blue), cf.board.get(4, 0));
		assert_eq!(Some(&Player::Red), cf.board.get(3, 0));
		assert_eq!(Some(&Player::Blue), cf.board.get(2, 0));
		assert_eq!(Some(&Player::Red), cf.board.get(1, 0));
		assert_eq!(Some(&Player::Blue), cf.board.get(0, 0));
	}

	#[test]
	fn test_emplace_col0_seven_times() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		for _ in 0..6 {
			assert!(cf.emplace(0));
		}
		assert_eq!(Player::Red, cf.turn);  // Is red's turn
		assert!(/* returns false */ !cf.emplace(0));  // Red tries to place, but is invalid
		assert_eq!(Player::Red, cf.turn); // Still red's turn
		/*
			   0 1 2 3 4 5 6
			0  B - - - - - -
			1  R - - - - - -
			2  B - - - - - -
			3  R - - - - - -
			4  B - - - - - -
			5  R - - - - - -
		*/
		assert_eq!(Some(&Player::Red), cf.board.get(5, 0));
		assert_eq!(Some(&Player::Blue), cf.board.get(4, 0));
		assert_eq!(Some(&Player::Red), cf.board.get(3, 0));
		assert_eq!(Some(&Player::Blue), cf.board.get(2, 0));
		assert_eq!(Some(&Player::Red), cf.board.get(1, 0));
		assert_eq!(Some(&Player::Blue), cf.board.get(0, 0));
	}

	#[test]
	fn test_get_winner_none() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();
		assert_eq!(None, cf.get_winner());
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
		assert_eq!(None, cf.get_winner());
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
		assert_eq!(None, cf.get_winner());
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
		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -   Red should win here.
			1  - - - - - - -
			2  R - - - - - -
			3  R B - - - - -
			4  R B - - - - -
			5  R B - - - - -
		*/
		assert_eq!(None, cf.get_winner());
		assert_eq!(GameState::Playing, cf.state);

		assert!(cf.emplace(0));  // R (2,0) victory

		assert_eq!(GameState::Won { player: Player::Red }, cf.state);
		assert_eq!(Some(Player::Red), cf.get_winner());
	}

	#[test]
	fn test_get_winner_5wide_red() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		assert!(cf.emplace(0));  // R (5,0)
		assert!(cf.emplace(0));  // B (4,0)
		assert!(cf.emplace(1));  // R (5,1)
		assert!(cf.emplace(1));  // B (4,1)
		assert!(cf.emplace(2));  // R (5,2)
		assert!(cf.emplace(2));  // B (4,2)
		assert!(cf.emplace(4));  // R (5,4)
		assert!(cf.emplace(4));  // B (4,4)
		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -   Red should win here.
			1  - - - - - - -
			2  - - - - - - -
			3  - - - - - - -
			4  B B B - B - -
			5  R R R R R - -
			         ^
			         |------- Place last
		*/
		assert_eq!(None, cf.get_winner());
		assert_eq!(GameState::Playing, cf.state);

		assert!(cf.emplace(3)); // R (5,3) victory

		assert_eq!(GameState::Won { player: Player::Red }, cf.state);
		assert_eq!(Some(Player::Red), cf.get_winner());
	}

	#[test]
	fn test_get_winner_none_tie() {
		let mut cf = ConnectFour::new(2, 1);
		cf.restart();

		assert!(cf.emplace(0));  // R (0,0)
		/*
			   0 1
			0  R B  Nobody wins here.
		*/
		assert_eq!(None, cf.get_winner());
		assert_eq!(GameState::Playing, cf.state);

		assert!(cf.emplace(1));  // B (0,1) draw

		assert_eq!(GameState::Closed, cf.state);
		assert_eq!(None, cf.get_winner());
	}

	#[test]
	fn test_get_winner_after_red_won() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		assert!(cf.emplace(0));  // R (5,0)
		assert!(cf.emplace(0));  // B (4,0)
		assert!(cf.emplace(1));  // R (5,1)
		assert!(cf.emplace(1));  // B (4,1)
		assert!(cf.emplace(2));  // R (5,2)
		assert!(cf.emplace(2));  // B (4,2)
		assert!(cf.emplace(4));  // R (5,4)
		assert!(cf.emplace(4));  // B (4,4)
		/*
			   0 1 2 3 4 5 6
			0  - - - - - - -
			1  - - - - - - -
			2  - - - - - - -
			3  - - - - - - -
			4  B B B X B - -  Blue should be blocked from playing, since red won.
			5  R R R R R - -
		*/
		assert_eq!(None, cf.get_winner());
		assert_eq!(GameState::Playing, cf.state);

		assert!(cf.emplace(3)); // R (5,3) victory

		assert_eq!(GameState::Won { player: Player::Red }, cf.state);
		assert_eq!(Some(Player::Red), cf.get_winner());

		assert!(/* returns false */ !cf.emplace(3)); // B (4,3) attempt after victory

		assert_eq!(GameState::Won { player: Player::Red }, cf.state);
		assert_eq!(Some(Player::Red), cf.get_winner());
	}

	#[test]
	fn test_get_count_in_direction() {
		let mut cf = ConnectFour::new(7, 6);
		cf.restart();

		cf.board.set(2, 1, Player::Red);
		cf.board.set(3, 1, Player::Red);
		cf.board.set(0, 2, Player::Red);
		cf.board.set(1, 2, Player::Red);
		cf.board.set(2, 2, Player::Red);
		cf.board.set(3, 2, Player::Red);
		cf.board.set(1, 3, Player::Red);
		cf.board.set(2, 3, Player::Blue);
		cf.board.set(3, 3, Player::Blue);
		cf.board.set(0, 4, Player::Red);
		cf.board.set(2, 4, Player::Red);
		/*
			   0 1 2 3 4
			0  - - R - R
			1  - - R R -
			2  - R R B R  <-- Note single BLUE piece on this line at (2,3)
			3  - R R B -  <-- and here at (3,3)
			4  - - - - -
		*/
		assert_eq!(3, cf.get_count_in_direction(2, 2, Direction::North));
		assert_eq!(3, cf.get_count_in_direction(2, 2, Direction::NorthEast));
		assert_eq!(1, cf.get_count_in_direction(2, 2, Direction::East));
		assert_eq!(1, cf.get_count_in_direction(2, 2, Direction::SouthEast));
		assert_eq!(2, cf.get_count_in_direction(2, 2, Direction::South));
		assert_eq!(2, cf.get_count_in_direction(2, 2, Direction::SouthWest));
		assert_eq!(2, cf.get_count_in_direction(2, 2, Direction::West));
		assert_eq!(1, cf.get_count_in_direction(2, 2, Direction::NorthWest));
	}
}
