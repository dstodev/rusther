use crate::commands::game_c4::c4::Direction;
use crate::commands::game_c4::index_from_rc::index_from_rc;

pub fn get_neighbor(index: usize, direction: Direction, board_width: usize, board_height: usize) -> Option<usize> {
	let stride = board_width;

	match direction {
		Direction::North => Some(index - stride),
		Direction::NorthEast => Some(index - stride + 1),
		Direction::East => Some(index + 1),
		Direction::SouthEast => Some(index + stride + 1),
		Direction::South => Some(index + stride),
		Direction::SouthWest => Some(index + stride - 1),
		Direction::West => Some(index - 1),
		Direction::NorthWest => Some(index - stride - 1),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_get_neighbor_middle() {
		let stride = 3;
		let index = index_from_rc(1, 1, stride);
		/*
			   0 1 2
			0  - - -
			1  - X -
			2  - - -
		*/
		assert_eq!(Some(index_from_rc(0, 1, stride)), get_neighbor(index, Direction::North, stride, stride));
		assert_eq!(Some(index_from_rc(0, 2, stride)), get_neighbor(index, Direction::NorthEast, stride, stride));
		assert_eq!(Some(index_from_rc(1, 2, stride)), get_neighbor(index, Direction::East, stride, stride));
		assert_eq!(Some(index_from_rc(2, 2, stride)), get_neighbor(index, Direction::SouthEast, stride, stride));
		assert_eq!(Some(index_from_rc(2, 1, stride)), get_neighbor(index, Direction::South, stride, stride));
		assert_eq!(Some(index_from_rc(2, 0, stride)), get_neighbor(index, Direction::SouthWest, stride, stride));
		assert_eq!(Some(index_from_rc(1, 0, stride)), get_neighbor(index, Direction::West, stride, stride));
		assert_eq!(Some(index_from_rc(0, 0, stride)), get_neighbor(index, Direction::NorthWest, stride, stride));
	}

	#[test]
	fn test_get_neighbor_top_left() {
		let stride = 3;
		let index = index_from_rc(0, 0, stride);
		/*
			   0 1 2
			0  X - -
			1  - - -
			2  - - -
		*/
		assert_eq!(None, get_neighbor(index, Direction::North, stride, stride));
		assert_eq!(None, get_neighbor(index, Direction::NorthEast, stride, stride));
		assert_eq!(Some(index_from_rc(1, 2, stride)), get_neighbor(index, Direction::East, stride, stride));
		assert_eq!(Some(index_from_rc(2, 2, stride)), get_neighbor(index, Direction::SouthEast, stride, stride));
		assert_eq!(Some(index_from_rc(2, 1, stride)), get_neighbor(index, Direction::South, stride, stride));
		assert_eq!(None, get_neighbor(index, Direction::SouthWest, stride, stride));
		assert_eq!(None, get_neighbor(index, Direction::West, stride, stride));
		assert_eq!(None, get_neighbor(index, Direction::NorthWest, stride, stride));
	}
}
