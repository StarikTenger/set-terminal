mod card;
mod display;
mod game;
mod players;

use std::io::{self, Write};
use std::time::{Duration, Instant};

use colored::Colorize;
use game::Game;

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn parse_selection(tokens: &[&str], board_len: usize) -> Result<[usize; 3], String> {
    let nums: Vec<i64> = tokens
        .iter()
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

struct Args {
    palette: display::Palette,
    /// `Some(names)` means team mode is active; `names` may be empty if
    /// `--players` was given with no names (a name is then prompted for).
    players: Option<Vec<String>>,
    highlight: bool,
    static_mode: bool,
}

const USAGE: &str = "\
Usage: set-game [OPTIONS]

A terminal implementation of the card game Set.

Options:
  --colors <red>,<green>,<purple>  Remap the terminal colors used for each card
                                    color. Accepts names (e.g. blue,yellow,cyan)
                                    or hex codes (e.g. #ff0000,#00ff00,#0000ff).
  --players [<names...>]           Start in team mode, optionally with an
                                    initial list of player names.
  --highlight=<true|false>         Highlight newly dealt cards (default: true).
  --static                         Static mode: claimed cards leave an empty
                                    gap instead of being replaced, and the
                                    board only refills once no Set remains.
  -h, --help                       Print this help and exit.";

fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut palette = display::Palette::default();
    let mut players: Option<Vec<String>> = None;
    let mut highlight = true;
    let mut static_mode = false;

    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "-h" | "--help" => {
                println!("{USAGE}");
                std::process::exit(0);
            }
            "--static" => {
                static_mode = true;
                i += 1;
            }
            "--colors" => {
                let Some(spec) = raw.get(i + 1) else {
                    eprintln!("--colors requires a value, e.g. --colors red,green,purple");
                    std::process::exit(1);
                };
                palette = display::parse_palette(spec).unwrap_or_else(|msg| {
                    eprintln!("{msg}");
                    std::process::exit(1);
                });
                i += 2;
            }
            "--players" => {
                let mut names = Vec::new();
                i += 1;
                while i < raw.len() && !raw[i].starts_with("--") {
                    names.push(raw[i].clone());
                    i += 1;
                }
                players = Some(names);
            }
            other => {
                if let Some(val) = other.strip_prefix("--highlight=") {
                    highlight = match val {
                        "true" => true,
                        "false" => false,
                        _ => {
                            eprintln!("--highlight expects true or false, got \"{val}\"");
                            std::process::exit(1);
                        }
                    };
                    i += 1;
                } else {
                    eprintln!("Unknown argument: {other}");
                    std::process::exit(1);
                }
            }
        }
    }

    Args { palette, players, highlight, static_mode }
}

/// Splits off a trailing player-name token from a claim's whitespace tokens.
/// Only applies in team mode, and only when the last token isn't itself a
/// number (so "1 5 9" still parses as three plain indices).
fn split_off_name<'a>(tokens: &'a [&'a str], team_mode: bool) -> (&'a [&'a str], Option<&'a str>) {
    if team_mode {
        if let Some((last, rest)) = tokens.split_last() {
            if last.parse::<i64>().is_err() {
                return (rest, Some(*last));
            }
        }
    }
    (tokens, None)
}

fn prompt_for_existing_player(roster: &players::Players) -> usize {
    loop {
        print!("Which player? ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            std::process::exit(1);
        }

        match roster.find_by_prefix(input.trim()) {
            Ok(idx) => return idx,
            Err(msg) => println!("{}", msg.yellow()),
        }
    }
}

fn prompt_for_player_name(roster: &mut players::Players) {
    loop {
        print!("Enter your name: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            std::process::exit(1);
        }

        match roster.register(input.trim()) {
            Ok(()) => break,
            Err(msg) => println!("{}", msg.yellow()),
        }
    }
}

fn main() {
    let args = parse_args();
    let palette = args.palette;
    let highlight = args.highlight;

    println!("{}", "=== SET ===".bold());
    println!(
        "Find 3 cards where, for every attribute (number, color, shape, shading),\n\
         all three cards match or all three differ."
    );
    println!(
        "Type 3 card numbers to claim a Set, or '/help' to see all in-game commands.\n"
    );
    if args.static_mode {
        println!(
            "{}",
            "Static mode: claimed cards leave a gap instead of being replaced; \
             the board only refills once no Set remains."
                .cyan()
        );
        println!();
    }

    let mut roster = players::Players::new();
    let mut team_mode = args.players.is_some();
    if let Some(initial_names) = args.players {
        for name in &initial_names {
            if let Err(msg) = roster.register(name) {
                eprintln!("{msg}");
                std::process::exit(1);
            }
        }
        if roster.is_empty() {
            prompt_for_player_name(&mut roster);
        }
        println!(
            "{}",
            format!("Team mode: {}", roster.names().join(", ")).cyan().bold()
        );
        println!(
            "{}",
            "Append a name (or a prefix of one) to your claim, e.g. \"1 5 9 al\" — \
             or leave it off and you'll be asked who's claiming."
                .dimmed()
        );
        println!();
    }

    let mut game = Game::new(args.static_mode);
    let start_time = Instant::now();
    // Only advances when a Set is actually found — wrong guesses don't end a round.
    let mut last_round_time = Instant::now();
    let mut set_durations: Vec<Duration> = Vec::new();

    loop {
        println!(
            "{}",
            format!(
                "Round {} — {} card(s) left in the deck",
                game.found_sets + 1,
                game.deck.len()
            )
            .dimmed()
        );
        display::render_board(&game.board, &palette, &game.new_indices, highlight);

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
            println!("{}", "Commands:".cyan());
            println!("  <n> <n> <n>       claim a Set, e.g. \"1 5 9\"");
            println!("  /count            show how many Sets are on the board");
            println!("  /cheat            reveal every Set on the board");
            println!("  /addplayer <name> register a player (activates team mode)");
            println!("  /finish, q        end the game and show your score");
            println!();
            continue;
        }
        if input.eq_ignore_ascii_case("/count") {
            let n = card::count_sets(&game.board);
            println!(
                "{}",
                format!("There {} {} Set{} on the board.", 
                        if n == 1 { "is" } else { "are" }, 
                        n,
                        if n == 1 { "" } else { "s" }, 
                        ).cyan()
            );
            println!();
            continue;
        }
        if input.eq_ignore_ascii_case("/cheat") {
            let sets = card::find_all_sets(&game.board);
            let n = sets.len();
            if sets.is_empty() {
                println!("{}", "There are no Sets on the board".cyan());
            } else {
            println!(
                "{}",
                format!("There {} {} Set{} on the board:", 
                        if n == 1 { "is" } else { "are" }, 
                        n,
                        if n == 1 { "" } else { "s" }, 
                        ).cyan()
            );
                for (i, j, k) in sets {
                    println!("  {} {} {}", i + 1, j + 1, k + 1);
                }
            }
            println!();
            continue;
        }
        {
            let mut parts = input.splitn(2, char::is_whitespace);
            if parts.next().is_some_and(|cmd| cmd.eq_ignore_ascii_case("/addplayer")) {
                let name = parts.next().unwrap_or("").trim();
                if name.is_empty() {
                    println!("{}", "Usage: /addplayer <name>".yellow());
                } else {
                    match roster.register(name) {
                        Ok(()) => {
                            let newly_activated = !team_mode;
                            team_mode = true;
                            if newly_activated {
                                println!(
                                    "{}",
                                    format!("Team mode activated. Added player: {name}").cyan().bold()
                                );
                            } else {
                                println!("{}", format!("Added player: {name}").cyan());
                            }
                        }
                        Err(msg) => println!("{}", msg.yellow()),
                    }
                }
                println!();
                continue;
            }
        }

        let tokens: Vec<&str> = input.split_whitespace().collect();
        let (number_tokens, name_token) = split_off_name(&tokens, team_mode);

        match parse_selection(number_tokens, game.board.len()) {
            Ok([i, j, k]) => {
                let player_idx = if team_mode {
                    Some(match name_token {
                        Some(name) => match roster.find_by_prefix(name) {
                            Ok(idx) => idx,
                            Err(msg) => {
                                println!("{}", msg.yellow());
                                println!();
                                continue;
                            }
                        },
                        None => prompt_for_existing_player(&roster),
                    })
                } else {
                    None
                };

                let claimed = game.try_claim(i, j, k);
                if claimed {
                    if let Some(idx) = player_idx {
                        roster.add_score(idx);
                    }
                }

                let attribution = player_idx
                    .map(|idx| format!(" ({})", roster.name(idx)))
                    .unwrap_or_default();
                if claimed {
                    println!("{}", format!("Nice, that's a Set!{attribution}").green().bold());
                } else {
                    println!("{}", format!("Not a Set — try again.{attribution}").red());
                }
                let now = Instant::now();
                println!(
                    "{}",
                    format!(
                        "Time since start: {} | since last round: {}",
                        format_duration(now - start_time),
                        format_duration(now - last_round_time)
                    )
                    .dimmed()
                );
                if claimed {
                    set_durations.push(now - last_round_time);
                    last_round_time = now;
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

    let total_time = Instant::now() - start_time;
    println!("{}", format!("Total time: {}", format_duration(total_time)).dimmed());
    if !set_durations.is_empty() {
        let avg = set_durations.iter().sum::<Duration>() / set_durations.len() as u32;
        let fastest = set_durations.iter().min().unwrap();
        let slowest = set_durations.iter().max().unwrap();
        println!(
            "{}",
            format!(
                "Average time per Set: {} (fastest: {}, slowest: {})",
                format_duration(avg),
                format_duration(*fastest),
                format_duration(*slowest)
            )
            .dimmed()
        );
    }

    if team_mode {
        println!("\n{}", "=== Scoreboard ===".bold());
        let name_width = roster.names().iter().map(|n| n.len()).max().unwrap_or(0);
        for (name, score) in roster.scoreboard() {
            let diamonds = "♦".repeat(score as usize);
            println!(
                "{:<width$}  {:>2}  {}",
                name,
                score,
                diamonds.magenta(),
                width = name_width
            );
        }
    }
}
