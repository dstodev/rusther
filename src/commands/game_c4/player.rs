use std::{
    fmt::{Display, Formatter},
    ops::Not,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Player {
    Red,
    Blue,
}

impl Default for Player {
    fn default() -> Self {
        Player::Red
    }
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let say = match self {
            Self::Red => 'R',
            Self::Blue => 'B',
        };
        write!(f, "{}", say)
    }
}

impl Not for Player {
    type Output = Player;
    fn not(self) -> Self::Output {
        match self {
            Player::Red => Player::Blue,
            Player::Blue => Player::Red,
        }
    }
}
