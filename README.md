# gomoku

A fast, zero-dependency game and rule engine for gomoku narabe (五目並べ) and its
major variants, written in Rust.

`gomoku` maintains board state and arbitrates the rules. It is an **engine, not a
player**.

`gomoku` is also my rust learning project. it probably doesn't follow best practices, or is good code in general. PR's are welcome.

## Highlights

- **Six rule variants** out of the box: Freestyle, Standard, Renju, Caro, Omok, Pente.
- **Full Renju forbidden-move detection** for Black — double-three, double-four, and
  overline — using the standard recursive definition of a live three.
- **Pente custodial captures**, capture counting, and capture-to-win.
- **Opening protocols**: Free, Pro, Long Pro, Swap, Swap2, and Yamaguchi, backed by a
  catalog of all **26 canonical Renju openings**.
- **Configurable board size** up to 19×19.
- **Serialization** with no `serde`: a FEN-like board snapshot and a replayable move list.
- **Zero runtime dependencies**, no `unsafe`, and fully `Send`/`Sync`.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
gomoku = { git = "https://github.com/lkse/gomoku" }
```

## Quick start

```rust
use gomoku::{Game, Player, Point, RuleSet, Status};

let mut game = Game::new(RuleSet::standard());

// Black builds a five along a row while White answers elsewhere.
for x in 0..4 {
    game.play(Point::new(x, 7)).unwrap(); // Black
    game.play(Point::new(x, 0)).unwrap(); // White
}
let outcome = game.play(Point::new(4, 7)).unwrap();

assert_eq!(outcome.status, Status::Win(Player::Black));
assert_eq!(game.winner(), Some(Player::Black));
```

Core operations: `play`, `undo`, `is_legal`, `legal_moves`, `status`, `winner`, and
`to_move`. Captures are reported on the returned `MoveOutcome`; capture totals are
available via `captures(player)`.

## Rule variants

Start from a preset and adjust its public fields as needed; resize any preset with
`.with_board_size(n)`. `RuleSet` is `#[non_exhaustive]`, so presets are the entry point
rather than struct literals. Validate an assembled configuration with
`RuleSet::validate`, or construct a game fallibly with `Game::try_new` (use `Game::new`
when the rules are known-good and a panic on misconfiguration is acceptable).

| Preset | Win condition | Notes |
|--------|---------------|-------|
| `RuleSet::freestyle()` | five **or more** in a row | no restrictions |
| `RuleSet::standard()`  | **exactly** five | an overline does not win |
| `RuleSet::renju()`     | exactly five | Black forbidden moves; free opening |
| `RuleSet::caro()`      | five or more | must not be blocked at both ends |
| `RuleSet::omok()`      | exactly five | 19×19 board |
| `RuleSet::pente()`     | five in a row, or five captured pairs | custodial captures, 19×19 |

## Opening protocols

Openings that involve interactive decisions are driven through a small state machine.
Query `game.opening_action()` to learn what is expected next, then call `play` (for
placements) or the matching decision method: `choose_color`, `swap2_decision`,
`announce_fifth_count`, `propose_fifths`, or `choose_fifth`.

```rust
use gomoku::{Game, OpeningAction, Point, RuleSet, Swap2Choice};

let mut game = Game::new(RuleSet::swap2());

// Place the opening three stones via `play`.
for &(x, y) in &[(7, 7), (7, 8), (8, 7)] {
    game.play(Point::new(x, y)).unwrap();
}

assert_eq!(game.opening_action(), OpeningAction::Swap2Decision);
game.swap2_decision(Swap2Choice::PlayWhite).unwrap();
```

Presets with openings: `renju_yamaguchi()`, `swap()`, `swap2()`, `pro()`, `long_pro()`.
The 26 named openings are enumerated in `OpeningName` / `ALL_OPENINGS`, each mapping to a
concrete three-stone placement (`placements`) with a reverse lookup (`identify`).

## Serialization

```rust
use gomoku::{Game, RuleSet};

let game = Game::from_move_list(RuleSet::standard(), "h8 h1 i8 i1 j8").unwrap();
let fen  = game.board().to_fen();   // "15:.../..."  rows top-first
let list = game.to_move_list();     // "h8 h1 i8 i1 j8"
```

Move-list replay supports placement-only openings (Free, Free Renju, Pro, Long Pro);
the interactive protocols require their decision methods.

## Performance

Occupancy lives in per-color bitboards, and because only the last stone can complete a
line, win/threat detection scans just the four axes through it — O(1) per move regardless
of board size. Representative single-move cost (`play` + `undo`, Criterion medians,
15×15):

| Variant | Per move | Variant | Per move |
|---------|---------:|---------|---------:|
| freestyle | ~25 ns | caro  | ~25 ns |
| standard  | ~25 ns | omok  | ~25 ns |
| pente     | ~51 ns | renju | ~278 ns |

Renju is the outlier: every Black move runs the recursive forbidden-move analysis. Whole-
board `legal_moves` generation is ~1 µs for most variants and ~120 µs for Renju.

Reproduce with:

```sh
cargo run --release --example bench   # self-play survey: throughput + outcomes
cargo bench                           # rigorous per-move distributions (Criterion)
```

## Project layout

```
src/
  lib.rs        crate root, public re-exports, lint policy
  stone.rs      Player / Cell
  point.rs      Point and coordinate parsing
  board.rs      bitboard storage, access, rendering
  game.rs       Game state, turn order, legality, history, openings
  error.rs      MoveError / ForbiddenKind
  serialize.rs  FEN + move-list (de)serialization
  rules/        win.rs, renju.rs, capture.rs, opening.rs, RuleSet presets
tests/          one integration suite per ruleset
examples/       demo.rs, bench.rs
benches/        engine.rs (Criterion)
```

## Development

```sh
cargo test                                   # unit + integration tests
cargo test --doc                             # documentation examples
cargo clippy --all-targets -- -D warnings    # lint
cargo fmt                                    # format
cargo doc --no-deps --open                   # API documentation
```

The library enforces `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`.
