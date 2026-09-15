use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::card::{self, Card};

const INITIAL_BOARD_SIZE: usize = 12;

pub struct Game {
    pub deck: Vec<Card>,
    pub board: Vec<Option<Card>>,
    pub found_sets: u32,
    pub attempts: u32,
    /// Board indices that were dealt or replaced during the most recent
    /// round (including the initial deal), for highlighting new cards.
    pub new_indices: Vec<usize>,
    /// In static mode, claimed cards leave an empty gap at their position
    /// instead of being replaced, and the board only refills (all gaps at
    /// once) once no Set remains among the cards still in play.
    static_mode: bool,
}

impl Game {
    pub fn new(static_mode: bool) -> Self {
        let mut deck = card::full_deck();
        deck.shuffle(&mut thread_rng());

        let mut game = Game {
            deck,
            board: Vec::new(),
            found_sets: 0,
            attempts: 0,
            new_indices: Vec::new(),
            static_mode,
        };
        game.deal(INITIAL_BOARD_SIZE);
        game.ensure_set_exists();
        game.new_indices = (0..game.board.len()).collect();
        game
    }

    fn deal(&mut self, n: usize) {
        for _ in 0..n {
            match self.deck.pop() {
                Some(c) => self.board.push(Some(c)),
                None => break,
            }
        }
    }

    /// Deals a new card into every empty slot, in place, stopping if the
    /// deck runs out. Returns the positions that were filled.
    fn refill_empty_slots(&mut self) -> Vec<usize> {
        let mut filled = Vec::new();
        for idx in 0..self.board.len() {
            if self.board[idx].is_none() {
                match self.deck.pop() {
                    Some(card) => {
                        self.board[idx] = Some(card);
                        filled.push(idx);
                    }
                    None => break,
                }
            }
        }
        filled
    }

    fn ensure_set_exists(&mut self) {
        while card::find_any_set(&self.board).is_none() && !self.deck.is_empty() {
            self.deal(3);
        }
    }

    pub fn is_over(&self) -> bool {
        self.deck.is_empty() && card::find_any_set(&self.board).is_none()
    }

    fn is_set_at(&self, i: usize, j: usize, k: usize) -> bool {
        match (self.board[i], self.board[j], self.board[k]) {
            (Some(a), Some(b), Some(c)) => card::is_set(&a, &b, &c),
            _ => false,
        }
    }

    /// Attempts to claim indices i, j, k (into `board`, already validated as
    /// in-bounds and distinct) as a Set. Returns true if it was a valid Set.
    pub fn try_claim(&mut self, i: usize, j: usize, k: usize) -> bool {
        self.attempts += 1;

        if !self.is_set_at(i, j, k) {
            return false;
        }

        self.found_sets += 1;

        let mut idxs = [i, j, k];
        idxs.sort_unstable();

        let mut new_indices = Vec::new();
        if self.static_mode {
            for &idx in &idxs {
                self.board[idx] = None;
            }
            // Don't deal new cards while a Set still exists among the
            // cards still in play; positions and numbering never shift.
            if card::find_any_set(&self.board).is_none() {
                new_indices.extend(self.refill_empty_slots());
                let before_len = self.board.len();
                while card::find_any_set(&self.board).is_none() && !self.deck.is_empty() {
                    self.deal(3);
                }
                new_indices.extend(before_len..self.board.len());
            }
        } else {
            if self.board.len() > INITIAL_BOARD_SIZE {
                // Board was temporarily expanded because no Set existed at
                // 12 cards; shrink it back down instead of dealing more.
                for &idx in idxs.iter().rev() {
                    self.board.remove(idx);
                }
            } else {
                for &idx in idxs.iter().rev() {
                    match self.deck.pop() {
                        Some(new_card) => {
                            self.board[idx] = Some(new_card);
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
        }

        self.new_indices = new_indices;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cards_in_play(game: &Game) -> usize {
        game.board.iter().filter(|c| c.is_some()).count() + game.deck.len()
    }

    fn run_to_completion(static_mode: bool) {
        for _ in 0..20 {
            let mut game = Game::new(static_mode);
            let mut iterations = 0;
            while !game.is_over() {
                iterations += 1;
                assert!(
                    iterations < 200,
                    "game did not terminate within a reasonable number of rounds"
                );

                let (i, j, k) = card::find_any_set(&game.board)
                    .expect("is_over() was false, so a Set must exist on the board");

                let cards_before = cards_in_play(&game);
                assert!(game.try_claim(i, j, k), "find_any_set returned a non-Set");
                let cards_after = cards_in_play(&game);
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

    #[test]
    fn playing_to_completion_never_panics_and_terminates() {
        run_to_completion(false);
    }

    #[test]
    fn playing_to_completion_in_static_mode_never_panics_and_terminates() {
        run_to_completion(true);
    }

    #[test]
    fn static_mode_never_shrinks_or_reorders_the_board() {
        let mut game = Game::new(true);
        let (i, j, k) = card::find_any_set(&game.board).unwrap();
        let claimed_cards = [game.board[i], game.board[j], game.board[k]];
        let other_cards: Vec<Option<Card>> = game
            .board
            .iter()
            .enumerate()
            .filter(|(idx, _)| ![i, j, k].contains(idx))
            .map(|(_, c)| *c)
            .collect();
        let board_len_before = game.board.len();

        assert!(game.try_claim(i, j, k));

        // Positions/numbering never shift: the board never shrinks (it may
        // only grow, exactly like the initial deal, if no Set exists even
        // after refilling), the claimed slots no longer hold their old
        // cards (emptied, or refilled in place), and every other
        // already-dealt card stayed exactly put.
        assert!(game.board.len() >= board_len_before);
        assert_ne!([game.board[i], game.board[j], game.board[k]], claimed_cards);
        let remaining_other_cards: Vec<Option<Card>> = game
            .board
            .iter()
            .take(board_len_before)
            .enumerate()
            .filter(|(idx, _)| ![i, j, k].contains(idx))
            .map(|(_, c)| *c)
            .collect();
        assert_eq!(other_cards, remaining_other_cards);
    }
}
