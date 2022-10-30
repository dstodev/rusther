use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use super::{Direction, Token};

#[derive(Clone, Debug, PartialEq)]
pub struct Board<T> {
    width: i32,
    height: i32,
    data: HashMap<i32, Token<T>>,
}

impl<T> Default for Board<T> {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            data: HashMap::new(),
        }
    }
}

impl<T> Board<T> {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            data: HashMap::new(),
        }
    }
    pub fn set(&mut self, row: i32, column: i32, value: T) -> &mut Self {
        let index = self.rc_to_index(row, column);
        self.data.insert(index, Token::new(row, column, value));
        self
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn data(&self) -> &HashMap<i32, Token<T>> {
        &self.data
    }
    pub fn get_neighbor(&self, row: i32, column: i32, direction: Direction) -> Option<&Token<T>> {
        let (neighbor_row, neighbor_column) = match direction {
            Direction::North => (row - 1, column),
            Direction::NorthEast => (row - 1, column + 1),
            Direction::East => (row, column + 1),
            Direction::SouthEast => (row + 1, column + 1),
            Direction::South => (row + 1, column),
            Direction::SouthWest => (row + 1, column - 1),
            Direction::West => (row, column - 1),
            Direction::NorthWest => (row - 1, column - 1),
        };
        self.get(neighbor_row, neighbor_column)
    }
    pub fn get(&self, row: i32, column: i32) -> Option<&Token<T>> {
        let in_bounds = row >= 0 && row < self.height && column >= 0 && column < self.width;

        if in_bounds {
            let index = self.rc_to_index(row, column);
            self.data.get(&index)
        } else {
            None
        }
    }
    fn rc_to_index(&self, row: i32, column: i32) -> i32 {
        row * self.width + column
    }
}

impl<T> Board<T>
where
    T: PartialEq,
{
    pub fn count_in_direction(&self, row: i32, column: i32, direction: Direction) -> i32 {
        let mut count = 0;

        if let Some(lhs) = self.get(row, column) {
            count += 1;

            if let Some(rhs) = self.get_neighbor(row, column, direction) {
                if rhs.value == lhs.value {
                    count += self.count_in_direction(rhs.row, rhs.column, direction)
                }
            }
        }
        count
    }
    pub fn count_in_bidirection(&self, row: i32, column: i32, direction: Direction) -> i32 {
        self.count_in_direction(row, column, direction)
            + self.count_in_direction(row, column, !direction)
            - 1 // Both calls add 1 for the token at (row,column)
    }
}

impl<T> Board<T>
where
    T: Clone,
{
    #[allow(dead_code)]
    pub fn fill(&mut self, value: T) {
        for row in 0..self.height {
            for column in 0..self.width {
                self.set(row, column, value.clone());
            }
        }
    }
}

impl<T> Display for Board<T>
where
    T: Display,
{
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
                    say += &format!("{} ", v.value);
                } else {
                    say += "- ";
                }
            }
            say += "\n";
        }
        write!(f, "{}", say)
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
    }

    #[test]
    fn fill() {
        let mut board = Board::<i32>::new(7, 6);
        board.fill(1);
        assert!(board
            .data
            .iter()
            .all(|(&k, v)| v.value == 1 && k == board.rc_to_index(v.row, v.column)));
    }

    #[test]
    fn get() {
        let mut board = Board::<i32>::new(1, 1);
        board.fill(1);
        assert_eq!(1, board.get(0, 0).unwrap().value);
    }

    #[test]
    fn set() {
        let mut board = Board::<i32>::new(1, 1);
        assert!(board.get(0, 0).is_none());
        board.set(0, 0, 1);
        assert_eq!(1, board.get(0, 0).unwrap().value);
    }

    #[test]
    fn set_chain() {
        let mut board = Board::<i32>::new(2, 1);
        board.set(0, 0, 1).set(0, 1, 2);
        assert_eq!(1, board.get(0, 0).unwrap().value);
        assert_eq!(2, board.get(0, 1).unwrap().value);
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
        assert_eq!(4, board.rc_to_index(1, 1));
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
        assert_eq!(
            Some(&Token::new(0, 1, ())),
            board.get_neighbor(1, 1, Direction::North)
        );
        assert_eq!(
            Some(&Token::new(0, 2, ())),
            board.get_neighbor(1, 1, Direction::NorthEast)
        );
        assert_eq!(
            Some(&Token::new(1, 2, ())),
            board.get_neighbor(1, 1, Direction::East)
        );
        assert_eq!(
            Some(&Token::new(2, 2, ())),
            board.get_neighbor(1, 1, Direction::SouthEast)
        );
        assert_eq!(
            Some(&Token::new(2, 1, ())),
            board.get_neighbor(1, 1, Direction::South)
        );
        assert_eq!(
            Some(&Token::new(2, 0, ())),
            board.get_neighbor(1, 1, Direction::SouthWest)
        );
        assert_eq!(
            Some(&Token::new(1, 0, ())),
            board.get_neighbor(1, 1, Direction::West)
        );
        assert_eq!(
            Some(&Token::new(0, 0, ())),
            board.get_neighbor(1, 1, Direction::NorthWest)
        );
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
        assert_eq!(
            Some(&Token::new(0, 1, ())),
            board.get_neighbor(0, 0, Direction::East)
        );
        assert_eq!(
            Some(&Token::new(1, 1, ())),
            board.get_neighbor(0, 0, Direction::SouthEast)
        );
        assert_eq!(
            Some(&Token::new(1, 0, ())),
            board.get_neighbor(0, 0, Direction::South)
        );
        assert_eq!(None, board.get_neighbor(0, 0, Direction::SouthWest));
        assert_eq!(None, board.get_neighbor(0, 0, Direction::West));
        assert_eq!(None, board.get_neighbor(0, 0, Direction::NorthWest));
    }

    #[derive(PartialEq)]
    enum Player {
        Red,
        Blue,
    }

    #[test]
    fn test_count_in_direction() {
        let mut board = Board::new(5, 5);

        board
            .set(2, 1, Player::Red)
            .set(3, 1, Player::Red)
            .set(0, 2, Player::Red)
            .set(1, 2, Player::Red)
            .set(2, 2, Player::Red)
            .set(3, 2, Player::Red)
            .set(1, 3, Player::Red)
            .set(2, 3, Player::Blue)
            .set(3, 3, Player::Blue)
            .set(0, 4, Player::Red)
            .set(2, 4, Player::Red);

        /*
               0 1 2 3 4
            0  - - R - R
            1  - - R R -
            2  - R R B R  <-- Test focuses on the center R at (2,2)
            3  - R R B -
            4  - - - - -  <-- and at the bottom left (4,0)
        */
        assert_eq!(3, board.count_in_direction(2, 2, Direction::North));
        assert_eq!(3, board.count_in_direction(2, 2, Direction::NorthEast));
        assert_eq!(1, board.count_in_direction(2, 2, Direction::East));
        assert_eq!(1, board.count_in_direction(2, 2, Direction::SouthEast));
        assert_eq!(2, board.count_in_direction(2, 2, Direction::South));
        assert_eq!(2, board.count_in_direction(2, 2, Direction::SouthWest));
        assert_eq!(2, board.count_in_direction(2, 2, Direction::West));
        assert_eq!(1, board.count_in_direction(2, 2, Direction::NorthWest));

        // At the empty bottom-left, facing top-right
        assert_eq!(0, board.count_in_direction(4, 0, Direction::NorthEast));
    }

    #[test]
    fn test_count_in_bidirection() {
        let mut board = Board::new(5, 5);

        board
            .set(2, 1, Player::Red)
            .set(3, 1, Player::Red)
            .set(0, 2, Player::Red)
            .set(1, 2, Player::Red)
            .set(2, 2, Player::Red)
            .set(3, 2, Player::Red)
            .set(1, 3, Player::Red)
            .set(2, 3, Player::Blue)
            .set(3, 3, Player::Blue)
            .set(0, 4, Player::Red)
            .set(2, 4, Player::Red);

        /*
               0 1 2 3 4
            0  - - R - R
            1  - - R R -
            2  - R R B R  <-- Test focuses on the center R at (2,2)
            3  - R R B -
            4  - - - - -  <-- and at the bottom left (4,0)
        */
        assert_eq!(4, board.count_in_bidirection(2, 2, Direction::North));
        assert_eq!(4, board.count_in_bidirection(2, 2, Direction::NorthEast));
        assert_eq!(2, board.count_in_bidirection(2, 2, Direction::East));
        assert_eq!(1, board.count_in_bidirection(2, 2, Direction::SouthEast));
        assert_eq!(4, board.count_in_bidirection(2, 2, Direction::South));
        assert_eq!(4, board.count_in_bidirection(2, 2, Direction::SouthWest));
        assert_eq!(2, board.count_in_bidirection(2, 2, Direction::West));
        assert_eq!(1, board.count_in_bidirection(2, 2, Direction::NorthWest));

        // At the empty bottom-left, facing top-right
        assert_eq!(0, board.count_in_direction(4, 0, Direction::NorthEast));
    }
}
