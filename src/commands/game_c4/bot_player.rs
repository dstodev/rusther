use super::{Board, Player};

/// Objects which derive `BotPlayer` expect `accept()` to be called followed by `decide()`.
pub trait BotPlayer {
    /// Accept the board state and who to play as, then decide which column to place a token in.
    fn choose_column(&mut self, board: &Board<Player>, player: Player) -> i32;
}
