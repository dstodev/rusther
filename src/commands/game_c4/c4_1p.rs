use super::{Board, BotPlayer, ConnectFour, ConnectFour2p, GameStatus, Player, RandomPlayer};

pub struct ConnectFour1p {
    game: ConnectFour2p,
    bot: Option<Box<dyn BotPlayer + Send + Sync>>,
}

impl ConnectFour1p {
    pub fn new(
        width: i32,
        height: i32,
        bot_player: Option<Box<dyn BotPlayer + Send + Sync + 'static>>,
    ) -> Self {
        let bot = bot_player.unwrap_or(Box::new(RandomPlayer));
        Self {
            game: ConnectFour2p::new(width, height),
            bot: Some(bot),
        }
    }
}

impl ConnectFour for ConnectFour1p {
    fn board(&self) -> &Board<Player> {
        self.game.board()
    }
    fn state(&self) -> GameStatus {
        self.game.state()
    }
    fn turn(&self) -> &Player {
        self.game.turn()
    }
    fn close(&mut self) {
        self.game.close()
    }
    fn emplace(&mut self, column: i32) -> bool {
        // Emplace player's decision ...
        if !self.game.emplace(column) {
            return false;
        }
        if self.state() == GameStatus::Playing {
            if let Some(mut bot) = self.bot.take() {
                let decision = bot.choose_column(self.board(), *self.turn());
                self.bot = Some(bot);

                // ... then emplace bot's decision
                if !self.game.emplace(decision) {
                    self.close();
                    log::warn!("C4 bot made invalid decision!");
                }
            }
        }
        true
    }
    fn get_winner(&self) -> Option<Player> {
        self.game.get_winner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPlayer {
        decisions: Vec<i32>,
    }

    impl MockPlayer {
        fn new(decisions: Vec<i32>) -> Self {
            Self { decisions }
        }
    }

    impl BotPlayer for MockPlayer {
        fn choose_column(&mut self, _board: &Board<Player>, _player: Player) -> i32 {
            self.decisions.pop().unwrap()
        }
    }

    #[test]
    fn test_player_win() {
        let player = MockPlayer::new(vec![3, 2, 1, 0]);
        let mut cf = ConnectFour1p::new(7, 6, Some(Box::new(player)));
        /*
               0 1 2 3 4 5 6
            0  - - - - - - -
            1  - - - - - - -
            2  - - - - - - -
            3  - - - - - - -
            4  B B B - - - -
            5  R R R R - - -
        */
        assert!(cf.emplace(0));
        assert_eq!(Player::Red, cf.board().get(5, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(4, 0).unwrap().into());
        assert_eq!(2, cf.board().data().len());
        assert_eq!(&Player::Red, cf.turn());

        assert!(cf.emplace(1));
        assert_eq!(Player::Red, cf.board().get(5, 1).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(4, 1).unwrap().into());

        assert!(cf.emplace(2));
        assert_eq!(Player::Red, cf.board().get(5, 2).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(4, 2).unwrap().into());

        assert!(cf.emplace(3));
        assert_eq!(Player::Red, cf.board().get(5, 3).unwrap().into());
        assert_eq!(None, cf.board().get(4, 3));

        assert_eq!(
            GameStatus::Won {
                player: Player::Red
            },
            cf.state()
        );
        assert_eq!(Some(Player::Red), cf.get_winner());
    }

    #[test]
    fn test_bot_win() {
        let player = MockPlayer::new(vec![3, 3, 2, 1, 0]);
        let mut cf = ConnectFour1p::new(7, 6, Some(Box::new(player)));
        /*
               0 1 2 3 4 5 6
            0  - - - - - - -
            1  - - - - - - -
            2  - - - - - - -
            3  - - - - - - -
            4  B B B B R - -
            5  R R R B R - -
        */
        assert!(cf.emplace(0));
        assert_eq!(Player::Red, cf.board().get(5, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(4, 0).unwrap().into());
        assert_eq!(2, cf.board().data().len());
        assert_eq!(&Player::Red, cf.turn());

        assert!(cf.emplace(1));
        assert_eq!(Player::Red, cf.board().get(5, 1).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(4, 1).unwrap().into());

        assert!(cf.emplace(2));
        assert_eq!(Player::Red, cf.board().get(5, 2).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(4, 2).unwrap().into());

        assert!(cf.emplace(4));
        assert_eq!(Player::Red, cf.board().get(5, 4).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(5, 3).unwrap().into());

        assert!(cf.emplace(4));
        assert_eq!(Player::Red, cf.board().get(4, 4).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(4, 3).unwrap().into());

        assert_eq!(
            GameStatus::Won {
                player: Player::Blue
            },
            cf.state()
        );
        assert_eq!(Some(Player::Blue), cf.get_winner());
    }

    #[test]
    fn test_player_invalid_move() {
        let player = MockPlayer::new(vec![2, 1]);
        let mut cf = ConnectFour1p::new(7, 1, Some(Box::new(player)));
        /*
               0 1 2 3 4 5 6
            0  R B - - - - -
        */
        assert!(cf.emplace(0));
        assert_eq!(Player::Red, cf.board().get(0, 0).unwrap().into());
        assert_eq!(Player::Blue, cf.board().get(0, 1).unwrap().into());
        assert_eq!(2, cf.board().data().len());

        assert_eq!(false, cf.emplace(0));
        assert_eq!(&Player::Red, cf.turn()); // Still red's turn
        assert_eq!(2, cf.board().data().len()); // Board has not changed
        assert_eq!(GameStatus::Playing, cf.state()); // Game is still active
    }

    #[test]
    fn test_bot_invalid_move() {
        let player = MockPlayer::new(vec![0, 0]);
        let mut cf = ConnectFour1p::new(7, 1, Some(Box::new(player)));
        /*
               0 1 2 3 4 5 6
            0  R - - - - - -
        */
        assert!(cf.emplace(0));
        assert_eq!(GameStatus::Closed, cf.state()); // Game entered an invalid state so is closed
        assert_eq!(None, cf.get_winner());
    }
}
