#[path = "../src/card.rs"]
mod card;
#[path = "../src/game.rs"]
mod game;
#[path = "../src/players.rs"]
mod players;

use game::Game;

/// Reproduces the claim-handling logic in main()'s team-mode branch: resolve
/// the claiming player, attempt the claim, and award the score on success.
#[test]
fn claiming_a_set_in_team_mode_awards_score_to_correct_player() {
    let mut roster = players::Players::new();
    roster.register("Alice").unwrap();
    roster.register("Bob").unwrap();

    let mut game = Game::new();
    let (i, j, k) = card::find_any_set(&game.board).expect("board always has a set at start");

    let idx = roster.find_by_prefix("al").unwrap();
    let claimed = game.try_claim(i, j, k);
    assert!(claimed);
    roster.add_score(idx);

    assert_eq!(roster.score(idx), 1);
    assert_eq!(roster.score(roster.find_by_prefix("bob").unwrap()), 0);
    assert_eq!(roster.scoreboard(), vec![("Alice", 1), ("Bob", 0)]);
}

#[test]
fn failed_claim_does_not_award_score() {
    let mut roster = players::Players::new();
    roster.register("Alice").unwrap();

    let mut game = Game::new();
    // Indices 0,1,2 are only a Set by chance; find two indices that are
    // guaranteed not to form one instead.
    let (i, j, _) = card::find_any_set(&game.board).unwrap();
    // Pick a third index that breaks the Set (any index not in the found triple).
    let bad_k = (0..game.board.len())
        .find(|&x| x != i && x != j && !card::is_set(&game.board[i], &game.board[j], &game.board[x]))
        .expect("board of 12 has a non-set completion for some pair");

    let idx = roster.find_by_prefix("al").unwrap();
    let claimed = game.try_claim(i, j, bad_k);
    assert!(!claimed);
    if claimed {
        roster.add_score(idx);
    }

    assert_eq!(roster.score(idx), 0);
}
