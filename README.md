# SET

A terminal implementation of the card game [Set](https://en.wikipedia.org/wiki/Set_(card_game)).

Find 3 cards where, for every attribute (number, color, shape, shading), all three cards match or all three differ.

## Run

```
cargo run
```

## Play

- Type 3 card numbers (e.g. `1 5 9`) to claim a Set.
- `/count` — show how many Sets are currently on the board.
- `/cheat` — reveal every Set currently on the board.
- `/addplayer <name>` — register a player (activates team mode if it isn't already on).
- `/finish` or `q` — end the game and show your score.

Each round shows the round number, cards left in the deck, and time since the game started and since the last round.

## Team mode

Start with `--players <names...>` to track individual scores, e.g.:

```
cargo run -- --players Alice Bob Claire
```

- If `--players` is given with no names, you'll be prompted for one.
- Append a name (or an unambiguous prefix) to a claim to attribute it, e.g. `1 5 9 al` — names are matched case-insensitively. Leave it off and you'll be asked who's claiming.
- Add more players anytime with `/addplayer <name>`.
- At game end, a scoreboard shows each player's score as a number and as ♦ diamonds.

## Options

- `--colors <red>,<green>,<purple>` — remap the terminal colors used for each card color. Accepts names (e.g. `--colors blue,yellow,cyan`) or hex codes (e.g. `--colors #ff0000,#00ff00,#0000ff`).
- `--highlight=false` — turn off highlighting of newly dealt cards (bold yellow `[n]` labels). Highlighting is on by default.

## Test

```
cargo test
```
