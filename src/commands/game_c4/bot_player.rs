use super::board::Board;

/// Objects which derive `BotPlayer` expect `accept()` to be called followed by `decide()`.
pub trait BotPlayer {
    type Token;

    /// Accept the board state
    fn accept(&mut self, board: &Board<Self::Token>);

    /// Decide which column to place a token in
    fn decide(&self) -> i32;
}
