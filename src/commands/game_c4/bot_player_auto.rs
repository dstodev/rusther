use super::bot_player::BotPlayer;
use super::{Board, Player};

struct AutoPlayer {
    player: Player,
    board: Board<Player>,
}
impl AutoPlayer {
    fn new(player: Player) -> Self {
        Self {
            player,
            board: Board::default(),
        }
    }
}

impl BotPlayer for AutoPlayer {
    type Token = Player;

    fn accept(&mut self, board: &Board<Self::Token>) {
        let board = board.clone();
        self.board = board;
    }
    fn decide(&self) -> i32 {
        0
    }
}
