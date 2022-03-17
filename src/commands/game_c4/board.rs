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
	row: i32,
	column: i32,
	value: T,
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

pub struct Board<T> {
	width: i32,
	height: i32,
	data: Vec<Option<Token<T>>>,
}

impl<T> Board<T> where T: Clone + PartialEq /* TODO: Limit scope of these traits? */ {
	pub fn new(width: i32, height: i32) -> Self {
		Self {
			width,
			height,
			data: vec![None; (width * height) as usize],
		}
	}
	pub fn fill(&mut self, value: T) {
		for row in 0..self.height {
			for column in 0..self.width {
				self.set(row, column, value.clone());
			}
		}
	}
	pub fn get_width(&self) -> i32 {
		self.width
	}
	pub fn get_height(&self) -> i32 {
		self.height
	}
	pub fn get_count_in_direction(&self, row: i32, column: i32, direction: Direction) -> i32 {
		if let Some(lhs) = self.get(row, column) {
			if let Some(rhs) = self.get_neighbor(row, column, direction) {
				if lhs.value == rhs.value {
					return 1 + self.get_count_in_direction(rhs.row, rhs.column, direction);
				}
			}
			1
		} else {
			0
		}
	}
	pub fn get_neighbor(&self, row: i32, column: i32, direction: Direction) -> Option<&Token<T>> {
		match direction {
			// @formatter:off
			Direction::North     => self.get(row - 1, column),
			Direction::NorthEast => self.get(row - 1, column + 1),
			Direction::East      => self.get(row,     column + 1),
			Direction::SouthEast => self.get(row + 1, column + 1),
			Direction::South     => self.get(row + 1, column),
			Direction::SouthWest => self.get(row + 1, column - 1),
			Direction::West      => self.get(row,     column - 1),
			Direction::NorthWest => self.get(row - 1, column - 1),
			// @formatter:on
		}
	}
	pub fn get(&self, row: i32, column: i32) -> Option<&Token<T>> {
		if let Some(index) = self.index_from_rc(row, column) {
			self.data[index as usize].as_ref()
		} else {
			None
		}
	}
	pub fn set(&mut self, row: i32, column: i32, value: T) {
		if let Some(index) = self.index_from_rc(row, column) {
			self.data[index as usize] = Some(Token::new(row, column, value));
		}
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
		assert_eq!(7, board.get_width());
		assert_eq!(6, board.get_height());
		assert_eq!(7 * 6, board.data.len());
	}

	#[test]
	fn fill() {
		let mut board = Board::<i32>::new(7, 6);
		board.fill(10);
		assert!(board.data.iter().all(|i| i.as_ref().unwrap().value == 10));
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
		let mut board = Board::<()>::new(stride, stride);
		board.set(0, 0, ());
		/*
			   0 1 2
			0  X - -
			1  - - -
			2  - - -
		*/
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
		assert_eq!(Some(&Token::new(0, 1, ())), board.get_neighbor(1, 1, Direction::North));
		assert_eq!(Some(&Token::new(0, 2, ())), board.get_neighbor(1, 1, Direction::NorthEast));
		assert_eq!(Some(&Token::new(1, 2, ())), board.get_neighbor(1, 1, Direction::East));
		assert_eq!(Some(&Token::new(2, 2, ())), board.get_neighbor(1, 1, Direction::SouthEast));
		assert_eq!(Some(&Token::new(2, 1, ())), board.get_neighbor(1, 1, Direction::South));
		assert_eq!(Some(&Token::new(2, 0, ())), board.get_neighbor(1, 1, Direction::SouthWest));
		assert_eq!(Some(&Token::new(1, 0, ())), board.get_neighbor(1, 1, Direction::West));
		assert_eq!(Some(&Token::new(0, 0, ())), board.get_neighbor(1, 1, Direction::NorthWest));
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
		assert_eq!(Some(&Token::new(0, 1, ())), board.get_neighbor(0, 0, Direction::East));
		assert_eq!(Some(&Token::new(1, 1, ())), board.get_neighbor(0, 0, Direction::SouthEast));
		assert_eq!(Some(&Token::new(1, 0, ())), board.get_neighbor(0, 0, Direction::South));
		assert_eq!(None, board.get_neighbor(0, 0, Direction::SouthWest));
		assert_eq!(None, board.get_neighbor(0, 0, Direction::West));
		assert_eq!(None, board.get_neighbor(0, 0, Direction::NorthWest));
	}
}
