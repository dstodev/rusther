use rand::prelude::SliceRandom;

use super::{Board, BotPlayer, Player};

pub struct RandomPlayer;

impl BotPlayer for RandomPlayer {
    fn choose_column(&mut self, board: &Board<Player>, _player: Player) -> i32 {
        let mut options: Vec<i32> = (0..board.width()).collect();

        // only keep columns which have space free (row 0 is the topmost row)
        options.retain(|&column| board.get(0, column).is_none());

        let column_decision = options.choose(&mut rand::thread_rng()).cloned();
        column_decision.unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_player_always_chooses_unblocked_spaces() {
        let mut player = RandomPlayer;

        for _ in 0..10 {
            let mut board = Board::<Player>::new(10, 1);
            /*
                   0 1 2 3  4 5 6 7 8 9
                0  - - - - - - - - - -
            */
            for _ in 0..10 {
                let decision = player.choose_column(&board, Player::Red);
                assert!(!board.data().contains_key(&decision));
                board.set(0, decision, Player::Red);
            }
        }
    }
}
