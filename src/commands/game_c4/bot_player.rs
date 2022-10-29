use super::board::Board;

pub trait BotPlayer {
    type Token;
    fn accept(&mut self, board: &Board<Self::Token>);
    fn decide(&self) -> i32;
}
