use super::{Board, Player};

pub trait BotPlayer {
    /// Accept the board state and who to play as, then decide which column to place a token in.
    fn choose_column(&mut self, board: &Board<Player>, player: Player) -> i32;
}
