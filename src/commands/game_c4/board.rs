use std::fmt::{Display, Formatter};

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

#[derive(Clone, Debug, PartialEq)]
pub struct Token<T> {
	pub row: i32,
	pub column: i32,
	pub value: T,
}

impl<T> Token<T> {
	pub fn new(row: i32, column: i32, value: T) -> Self {
		Self {
			row,
			column,
			value,
		}
	}
}

#[derive(Debug)]
pub struct Board<T> {
	width: i32,
	height: i32,
	data: Vec<Option<Token<T>>>,
}

impl<T> Display for Board<T> where T: Display {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut say = String::new();

		say += "   ";

		for column in 0..self.width {
			say += &format!("{} ", column);
		}
		say += "\n";

		for row in 0..self.height {
			say += &format!("{}  ", row);

			for column in 0..self.width {
				if let Some(v) = self.get(row, column) {
					say += &format!("{} ", v);
				} else {
					say += "- ";
				}
			}
			say += "\n";
		}
		write!(f, "{}", say)
	}
}

impl<T> Board<T> where T: Clone {
	pub fn new(width: i32, height: i32) -> Self {
		Self {
			width,
			height,
			data: vec![None; (width * height) as usize],
		}
	}
	#[allow(dead_code)]
	pub fn fill(&mut self, value: T) {
		for row in 0..self.height {
			for column in 0..self.width {
				self.set(row, column, value.clone());
			}
		}
	}
}

impl<T> Board<T> {
	pub fn width(&self) -> i32 {
		self.width
	}

	pub fn height(&self) -> i32 {
		self.height
	}

	pub fn data(&self) -> &Vec<Option<Token<T>>> {
		&self.data
	}

	pub fn get_neighbor(&self, row: i32, column: i32, direction: Direction) -> Option<Token<&T>> {
		let (row, column) = match direction {
			// @formatter:off
			Direction::North     => (row - 1, column),
			Direction::NorthEast => (row - 1, column + 1),
			Direction::East      => (row,     column + 1),
			Direction::SouthEast => (row + 1, column + 1),
			Direction::South     => (row + 1, column),
			Direction::SouthWest => (row + 1, column - 1),
			Direction::West      => (row,     column - 1),
			Direction::NorthWest => (row - 1, column - 1),
			// @formatter:on
		};
		if let Some(value) = self.get(row, column) {
			return Some(Token::new(row, column, value));
		}
		None
	}

	pub fn get(&self, row: i32, column: i32) -> Option<&T> {
		if let Some(index) = self.index_from_rc(row, column) {
			if let Some(Some(token)) = self.data.get(index as usize) {
				let value = &token.value;
				return Some(value);
			}
		}
		None
	}

	pub fn set(&mut self, row: i32, column: i32, value: T) -> &mut Self {
		if let Some(index) = self.index_from_rc(row, column) {
			self.data[index as usize] = Some(Token::new(row, column, value));
		}
		self
	}

	fn index_from_rc(&self, row: i32, column: i32) -> Option<i32> {
		if row >= 0 && row < self.height && column >= 0 && column < self.width {
			let stride = self.width;
			Some(row * stride + column)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new() {
		let board = Board::<()>::new(7, 6);
		assert_eq!(7, board.width());
		assert_eq!(6, board.height());
		assert_eq!(7 * 6, board.data().len());
		assert!(board.data.iter().all(|i| i.is_none()));
	}

	#[test]
	fn fill() {
		let mut board = Board::<i32>::new(7, 6);
		board.fill(1);
		assert!(board.data.iter().all(|i| i.as_ref().unwrap().value == 1));
	}

	#[test]
	fn get_() {
		let mut board = Board::<i32>::new(1, 1);
		board.fill(1);
		let first = board.get(0, 0);
		assert!(first.is_some());
		assert_eq!(&1, first.unwrap());
	}

	#[test]
	fn set() {
		let mut board = Board::<i32>::new(1, 1);
		assert!(board.get(0, 0).is_none());
		board.set(0, 0, 1);
		assert_eq!(&1, board.get(0, 0).unwrap());
	}

	#[test]
	fn set_chain() {
		let mut board = Board::<i32>::new(2, 1);
		board.set(0, 0, 1).set(0, 1, 2);
		assert_eq!(&1, board.get(0, 0).unwrap());
		assert_eq!(&2, board.get(0, 1).unwrap());
	}

	#[test]
	fn index_from_rc_middle() {
		let stride = 3;
		let board = Board::<()>::new(stride, stride);
		/*
			   0 1 2
			0  - - -
			1  - X -
			2  - - -
		*/
		assert_eq!(Some(4), board.index_from_rc(1, 1));
	}

	#[test]
	fn index_from_rc_out_of_bounds() {
		let stride = 3;
		let board = Board::<()>::new(stride, stride);
		assert_eq!(None, board.index_from_rc(-1, 0));
		assert_eq!(None, board.index_from_rc(0, -1));
		assert_eq!(Some(0), board.index_from_rc(0, 0));
	}

	#[test]
	fn test_get_neighbor_middle() {
		let stride = 3;
		let mut board = Board::<()>::new(stride, stride);
		board.fill(());
		/*
			   0 1 2
			0  - - -
			1  - X -
			2  - - -
		*/
		assert_eq!(Some(Token::new(0, 1, &())), board.get_neighbor(1, 1, Direction::North));
		assert_eq!(Some(Token::new(0, 2, &())), board.get_neighbor(1, 1, Direction::NorthEast));
		assert_eq!(Some(Token::new(1, 2, &())), board.get_neighbor(1, 1, Direction::East));
		assert_eq!(Some(Token::new(2, 2, &())), board.get_neighbor(1, 1, Direction::SouthEast));
		assert_eq!(Some(Token::new(2, 1, &())), board.get_neighbor(1, 1, Direction::South));
		assert_eq!(Some(Token::new(2, 0, &())), board.get_neighbor(1, 1, Direction::SouthWest));
		assert_eq!(Some(Token::new(1, 0, &())), board.get_neighbor(1, 1, Direction::West));
		assert_eq!(Some(Token::new(0, 0, &())), board.get_neighbor(1, 1, Direction::NorthWest));
	}

	#[test]
	fn test_get_neighbor_top_left() {
		let stride = 3;
		let mut board = Board::<()>::new(stride, stride);
		board.fill(());
		/*
			   0 1 2
			0  X - -
			1  - - -
			2  - - -
		*/
		assert_eq!(None, board.get_neighbor(0, 0, Direction::North));
		assert_eq!(None, board.get_neighbor(0, 0, Direction::NorthEast));
		assert_eq!(Some(Token::new(0, 1, &())), board.get_neighbor(0, 0, Direction::East));
		assert_eq!(Some(Token::new(1, 1, &())), board.get_neighbor(0, 0, Direction::SouthEast));
		assert_eq!(Some(Token::new(1, 0, &())), board.get_neighbor(0, 0, Direction::South));
		assert_eq!(None, board.get_neighbor(0, 0, Direction::SouthWest));
		assert_eq!(None, board.get_neighbor(0, 0, Direction::West));
		assert_eq!(None, board.get_neighbor(0, 0, Direction::NorthWest));
	}
}
