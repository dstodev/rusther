pub fn index_from_rc(row: usize, column: usize, stride: usize) -> usize {
	row * stride + column
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn index_from_rc_middle() {
		let stride = 3;
		let board = vec![(); stride * stride];
		/*
			   0 1 2
			0  - - -
			1  - X -
			2  - - -
		*/
		assert_eq!(4, index_from_rc(1, 1, stride));
	}
}
