use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::{self, Card};

const INITIAL_BOARD_SIZE: usize = 12;

pub struct Game {
    pub deck: Vec<Card>,
    pub board: Vec<Card>,
    pub found_sets: u32,
    pub attempts: u32,
    /// Board indices that were dealt or replaced during the most recent
    /// round (including the initial deal), for highlighting new cards.
    pub new_indices: Vec<usize>,
}

impl Game {
    pub fn new() -> Self {
        let mut deck = card::full_deck();
        deck.shuffle(&mut thread_rng());

        let mut game = Game {
            deck,
            board: Vec::new(),
            found_sets: 0,
            attempts: 0,
            new_indices: Vec::new(),
        };
        game.deal(INITIAL_BOARD_SIZE);
        game.ensure_set_exists();
        game.new_indices = (0..game.board.len()).collect();
        game
    }

    fn deal(&mut self, n: usize) {
        for _ in 0..n {
            match self.deck.pop() {
                Some(c) => self.board.push(c),
                None => break,
            }
        }
    }

    fn ensure_set_exists(&mut self) {
        while card::find_any_set(&self.board).is_none() && !self.deck.is_empty() {
            self.deal(3);
        }
    }

    pub fn is_over(&self) -> bool {
        self.deck.is_empty() && card::find_any_set(&self.board).is_none()
    }

    /// Attempts to claim indices i, j, k (into `board`, already validated as
    /// in-bounds and distinct) as a Set. Returns true if it was a valid Set.
    pub fn try_claim(&mut self, i: usize, j: usize, k: usize) -> bool {
        self.attempts += 1;

        if !card::is_set(&self.board[i], &self.board[j], &self.board[k]) {
            return false;
        }

        self.found_sets += 1;

        let mut idxs = [i, j, k];
        idxs.sort_unstable();

        let mut new_indices = Vec::new();
        if self.board.len() > INITIAL_BOARD_SIZE {
            // Board was temporarily expanded because no Set existed at 12
            // cards; shrink it back down instead of dealing more.
            for &idx in idxs.iter().rev() {
                self.board.remove(idx);
            }
        } else {
            for &idx in idxs.iter().rev() {
                match self.deck.pop() {
                    Some(new_card) => {
                        self.board[idx] = new_card;
                        new_indices.push(idx);
                    }
                    None => {
                        self.board.remove(idx);
                    }
                }
            }
        }

        let before_len = self.board.len();
        self.ensure_set_exists();
        new_indices.extend(before_len..self.board.len());
        self.new_indices = new_indices;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playing_to_completion_never_panics_and_terminates() {
        for _ in 0..20 {
            let mut game = Game::new();
            let mut iterations = 0;
            while !game.is_over() {
                iterations += 1;
                assert!(
                    iterations < 200,
                    "game did not terminate within a reasonable number of rounds"
                );

                let (i, j, k) = card::find_any_set(&game.board)
                    .expect("is_over() was false, so a Set must exist on the board");

                let cards_before = game.board.len() + game.deck.len();
                assert!(game.try_claim(i, j, k), "find_any_set returned a non-Set");
                let cards_after = game.board.len() + game.deck.len();
                assert_eq!(
                    cards_before - 3,
                    cards_after,
                    "claimed Sets should be removed from play, not recycled"
                );

                // An invalid claim (reusing indices 0,0,1) must never succeed or panic,
                // and must never mutate the board.
                if game.board.len() >= 2 {
                    let board_snapshot = game.board.clone();
                    let claimed = game.try_claim(0, 0, 1);
                    assert!(!claimed, "duplicate indices should never count as a Set");
                    assert_eq!(board_snapshot, game.board);
                }
            }
            assert!(card::find_any_set(&game.board).is_none());
        }
    }
}
