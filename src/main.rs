mod card;
mod display;
mod game;

use std::io::{self, Write};

use colored::Colorize;
use game::Game;

fn parse_selection(input: &str, board_len: usize) -> Result<[usize; 3], String> {
    let nums: Vec<i64> = input
        .split_whitespace()
        .map(|s| s.parse::<i64>())
        .collect::<Result<_, _>>()
        .map_err(|_| "Please enter 3 numbers, e.g. \"1 5 9\".".to_string())?;

    if nums.len() != 3 {
        return Err("Please enter exactly 3 numbers.".to_string());
    }

    let mut idxs = [0usize; 3];
    for (slot, &n) in idxs.iter_mut().zip(nums.iter()) {
        if n < 1 || n as usize > board_len {
            return Err(format!(
                "{n} is not a card on the board (pick 1-{board_len})."
            ));
        }
        *slot = (n - 1) as usize;
    }

    let mut sorted = idxs;
    sorted.sort_unstable();
    if sorted[0] == sorted[1] || sorted[1] == sorted[2] {
        return Err("Pick 3 different cards.".to_string());
    }

    Ok(idxs)
}

fn main() {
    println!("{}", "=== SET ===".bold());
    println!(
        "Find 3 cards where, for every attribute (number, color, shape, shading),\n\
         all three cards match or all three differ."
    );
    println!(
        "Type 3 card numbers to claim a Set, '/help' for hints, or '/finish' (or 'q') to end the game.\n"
    );

    let mut game = Game::new();

    loop {
        display::render_board(&game.board);

        if game.is_over() {
            break;
        }

        print!("\n> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();

        if input.eq_ignore_ascii_case("q")
            || input.eq_ignore_ascii_case("quit")
            || input.eq_ignore_ascii_case("/finish")
        {
            break;
        }
        if input.is_empty() {
            continue;
        }
        if input.eq_ignore_ascii_case("/help") {
            let n = card::count_sets(&game.board);
            println!(
                "{}",
                format!("There {} {} Set(s) on the board.", if n == 1 { "is" } else { "are" }, n).cyan()
            );
            println!();
            continue;
        }

        match parse_selection(input, game.board.len()) {
            Ok([i, j, k]) => {
                if game.try_claim(i, j, k) {
                    println!("{}", "Nice, that's a Set!".green().bold());
                } else {
                    println!("{}", "Not a Set — try again.".red());
                }
            }
            Err(msg) => println!("{}", msg.yellow()),
        }
        println!();
    }

    println!(
        "\n{} You found {} Set(s) in {} attempt(s). {} cards left in the deck.",
        "Game over.".bold(),
        game.found_sets,
        game.attempts,
        game.deck.len()
    );
}
