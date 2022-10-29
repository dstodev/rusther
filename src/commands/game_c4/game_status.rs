use super::Player;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GameStatus {
    Closed,
    Playing,
    Won { player: Player },
}
