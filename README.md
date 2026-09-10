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
- `/finish` or `q` — end the game and show your score.

## Options

- `--colors <red>,<green>,<purple>` — remap the terminal colors used for each card color. Accepts names (e.g. `--colors blue,yellow,cyan`) or hex codes (e.g. `--colors #ff0000,#00ff00,#0000ff`).

## Test

```
cargo test
```
