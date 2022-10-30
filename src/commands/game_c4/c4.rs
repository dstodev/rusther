use super::{Board, GameStatus, Player};

pub trait ConnectFour {
    fn board(&self) -> &Board<Player>;
    fn state(&self) -> GameStatus;
    fn turn(&self) -> &Player;
    fn close(&mut self);

    fn emplace(&mut self, column: i32) -> bool;
    fn get_winner(&self) -> Option<Player>;
}
