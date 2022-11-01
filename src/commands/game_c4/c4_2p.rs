use super::{Board, ConnectFour, Direction, GameStatus, Player};

#[derive(Clone, Debug)]
pub struct ConnectFour2p {
    turn: Player,
    state: GameStatus,
    board: Board<Player>,
    last_pos_r: i32,
    last_pos_c: i32,
}

impl ConnectFour2p {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            state: GameStatus::Playing,
            turn: Player::Red,
            board: Board::new(width, height),
            last_pos_r: 0,
            last_pos_c: 0,
        }
    }
}

impl ConnectFour for ConnectFour2p {
    fn board(&self) -> &Board<Player> {
        &self.board
    }
    fn state(&self) -> GameStatus {
        self.state
    }
    fn turn(&self) -> &Player {
        &self.turn
    }
    fn close(&mut self) {
        self.state = GameStatus::Closed;
    }
    fn emplace(&mut self, column: i32) -> bool {
        let valid_move =
            self.state == GameStatus::Playing && 0 <= column && column < self.board.width();

        if valid_move {
            for row in (0..self.board.height()).rev() {
                if self.board.get(row, column).is_none() {
                    self.board.set(row, column, self.turn);
                    self.last_pos_r = row;
                    self.last_pos_c = column;

                    if let Some(player) = self.get_winner() {
                        //self.board.fill(winner);  // Cool effect, but obscures the winning move
                        self.state = GameStatus::Won { player };
                    } else if self.board.data().len()
                        == self.board.width() as usize * self.board.height() as usize
                    {
                        // Board is full, but there are no winners. A draw!
                        self.state = GameStatus::Closed;
                    }
                    self.turn = !self.turn;
                    return true;
                }
            }
        }
        false
    }
    fn get_winner(&self) -> Option<Player> {
        if let GameStatus::Won { player } = self.state {
            return Some(player);
        }

        let row = self.last_pos_r;
        let column = self.last_pos_c;

        let n_s = self
            .board
            .count_in_bidirection(row, column, Direction::North);

        let ne_sw = self
            .board
            .count_in_bidirection(row, column, Direction::NorthEast);

        let e_w = self
            .board
            .count_in_bidirection(row, column, Direction::East);

        let se_nw = self
            .board
            .count_in_bidirection(row, column, Direction::NorthWest);

        let max = n_s.max(ne_sw).max(e_w).max(se_nw);

        if max >= 4 {
            Some(self.turn)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Token;
    use super::*;

    impl From<&Token<Player>> for Player {
        fn from(o: &Token<Player>) -> Self {
            o.value
        }
    }

    #[test]
    fn test_new_default() {
        let cf = ConnectFour2p::new(7, 6);
        assert_eq!(GameStatus::Playing, cf.state);
        assert_eq!(Player::Red, cf.turn);
        assert_eq!(7, cf.board.width());
        assert_eq!(6, cf.board.height());
    }

    #[test]
    fn test_emplace_col0() {
        let mut cf = ConnectFour2p::new(7, 6);
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
        assert_eq!(Player::Red, cf.board.get(5, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.turn);
    }

    #[test]
    fn test_emplace_col0_when_closed() {
        let mut cf = ConnectFour2p::new(7, 6);
        cf.state = GameStatus::Closed;
        assert_eq!(false, cf.emplace(0));
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
        let mut cf = ConnectFour2p::new(7, 6);
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
        assert_eq!(Player::Red, cf.board.get(5, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board.get(4, 0).unwrap().into());
        assert_eq!(Player::Red, cf.turn);
    }

    #[test]
    fn test_emplace_col6() {
        let mut cf = ConnectFour2p::new(7, 6);
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
        assert_eq!(Player::Red, cf.board.get(5, 6).unwrap().into());
        assert_eq!(Player::Blue, cf.turn);
    }

    #[test]
    fn test_emplace_col7_out_of_bounds() {
        let mut cf = ConnectFour2p::new(7, 6);
        assert_eq!(Player::Red, cf.turn);
        assert_eq!(false, cf.emplace(7));
        assert_eq!(Player::Red, cf.turn);
    }

    #[test]
    fn test_emplace_coln1_out_of_bounds() {
        let mut cf = ConnectFour2p::new(7, 6);
        assert_eq!(Player::Red, cf.turn);
        assert_eq!(false, cf.emplace(-1));
        assert_eq!(Player::Red, cf.turn);
    }

    #[test]
    fn test_emplace_col0_six_times() {
        let mut cf = ConnectFour2p::new(7, 6);

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
        assert_eq!(Player::Red, cf.board.get(5, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board.get(4, 0).unwrap().into());
        assert_eq!(Player::Red, cf.board.get(3, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board.get(2, 0).unwrap().into());
        assert_eq!(Player::Red, cf.board.get(1, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board.get(0, 0).unwrap().into());
    }

    #[test]
    fn test_emplace_col0_seven_times() {
        let mut cf = ConnectFour2p::new(7, 6);

        for _ in 0..6 {
            assert!(cf.emplace(0));
        }
        assert_eq!(Player::Red, cf.turn); // Is red's turn
        assert_eq!(false, cf.emplace(0)); // Red tries to place, but is invalid
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
        assert_eq!(Player::Red, cf.board.get(5, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board.get(4, 0).unwrap().into());
        assert_eq!(Player::Red, cf.board.get(3, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board.get(2, 0).unwrap().into());
        assert_eq!(Player::Red, cf.board.get(1, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board.get(0, 0).unwrap().into());
    }

    #[test]
    fn test_get_winner_none() {
        let cf = ConnectFour2p::new(7, 6);
        assert_eq!(None, cf.get_winner());
    }

    #[test]
    fn test_get_winner_4tall_mixed() {
        let mut cf = ConnectFour2p::new(7, 6);

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
        let mut cf = ConnectFour2p::new(7, 6);

        assert!(cf.emplace(0)); // R (5,0)
        assert!(cf.emplace(1)); // B (5,1)
        assert!(cf.emplace(0)); // R (4,0)
        assert!(cf.emplace(1)); // B (4,1)
        assert!(cf.emplace(0)); // R (3,0)

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
        let mut cf = ConnectFour2p::new(7, 6);

        assert!(cf.emplace(0)); // R (5,0)
        assert!(cf.emplace(1)); // B (5,1)
        assert!(cf.emplace(0)); // R (4,0)
        assert!(cf.emplace(1)); // B (4,1)
        assert!(cf.emplace(0)); // R (3,0)

        assert!(cf.emplace(1)); // B (3,1)

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
        assert_eq!(GameStatus::Playing, cf.state);

        assert!(cf.emplace(0)); // R (2,0) victory

        assert_eq!(
            GameStatus::Won {
                player: Player::Red
            },
            cf.state
        );
        assert_eq!(Some(Player::Red), cf.get_winner());
    }

    #[test]
    fn test_get_winner_5wide_red() {
        let mut cf = ConnectFour2p::new(7, 6);

        assert!(cf.emplace(0)); // R (5,0)
        assert!(cf.emplace(0)); // B (4,0)
        assert!(cf.emplace(1)); // R (5,1)
        assert!(cf.emplace(1)); // B (4,1)
        assert!(cf.emplace(2)); // R (5,2)
        assert!(cf.emplace(2)); // B (4,2)
        assert!(cf.emplace(4)); // R (5,4)
        assert!(cf.emplace(4)); // B (4,4)

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
        assert_eq!(GameStatus::Playing, cf.state);

        assert!(cf.emplace(3)); // R (5,3) victory

        assert_eq!(
            GameStatus::Won {
                player: Player::Red
            },
            cf.state
        );
        assert_eq!(Some(Player::Red), cf.get_winner());
    }

    #[test]
    fn test_get_winner_none_tie() {
        let mut cf = ConnectFour2p::new(2, 1);

        assert!(cf.emplace(0)); // R (0,0)

        /*
               0 1
            0  R B  Nobody wins here.
        */
        assert_eq!(None, cf.get_winner());
        assert_eq!(GameStatus::Playing, cf.state);

        assert!(cf.emplace(1)); // B (0,1) draw

        assert_eq!(GameStatus::Closed, cf.state);
        assert_eq!(None, cf.get_winner());
    }

    #[test]
    fn test_get_winner_after_red_won() {
        let mut cf = ConnectFour2p::new(7, 6);

        assert!(cf.emplace(0)); // R (5,0)
        assert!(cf.emplace(0)); // B (4,0)
        assert!(cf.emplace(1)); // R (5,1)
        assert!(cf.emplace(1)); // B (4,1)
        assert!(cf.emplace(2)); // R (5,2)
        assert!(cf.emplace(2)); // B (4,2)
        assert!(cf.emplace(4)); // R (5,4)
        assert!(cf.emplace(4)); // B (4,4)

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
        assert_eq!(GameStatus::Playing, cf.state);

        assert!(cf.emplace(3)); // R (5,3) victory

        assert_eq!(
            GameStatus::Won {
                player: Player::Red
            },
            cf.state
        );
        assert_eq!(Some(Player::Red), cf.get_winner());

        assert_eq!(false, cf.emplace(3)); // B (4,3) attempt after victory

        assert_eq!(
            GameStatus::Won {
                player: Player::Red
            },
            cf.state
        );
        assert_eq!(Some(Player::Red), cf.get_winner());
    }

    #[test]
    fn test_close() {
        let mut cf = ConnectFour2p::new(7, 6);

        assert_eq!(None, cf.get_winner());
        assert_eq!(GameStatus::Playing, cf.state);

        cf.close();

        assert_eq!(None, cf.get_winner());
        assert_eq!(GameStatus::Closed, cf.state);

        let board = cf.board().clone();
        assert_eq!(false, cf.emplace(0)); // attempt after close
        assert_eq!(cf.board(), &board);

        assert_eq!(None, cf.get_winner());
        assert_eq!(GameStatus::Closed, cf.state);
    }
}
