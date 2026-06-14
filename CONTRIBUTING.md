# Contributing to gomoku

Thanks for taking an interest! This is a personal learning project, so PRs,
issues, questions, and "here's a cleaner way to do this" notes are all genuinely
welcome, don't feel you need to polish something to perfection before opening it.

## Prerequisites

A stable Rust toolchain. The crate's minimum supported Rust version (MSRV) is
**1.70**; please don't reach for `std` APIs newer than that without flagging it.

```sh
rustup toolchain install stable --component rustfmt --component clippy
```

The library has **zero runtime dependencies** and forbids `unsafe`
(`#![forbid(unsafe_code)]`) - please keep both true. The only dev-dependency is
the Criterion-compatible benchmark harness.

## The checks CI runs

A pull request needs all of these to pass. They're quick; run them locally before
pushing:

```sh
cargo fmt --check                            # formatting
cargo clippy --all-targets -- -D warnings    # lints (warnings are errors)
cargo test --all-targets                     # unit + integration tests
cargo test --doc                             # documentation examples
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps   # docs build, no broken links
```

On Windows PowerShell, set the docs flag separately:

```powershell
$env:RUSTDOCFLAGS = "-D warnings"; cargo doc --no-deps
```

`cargo fmt` (without `--check`) fixes formatting for you.

## How the code is organized

```
src/
  lib.rs        crate root, public re-exports, lint policy
  stone.rs      Player / Cell
  point.rs      Point and coordinate parsing
  board.rs      bitboard storage, access, rendering
  game.rs       Game state, turn order, legality, history, openings
  error.rs      MoveError / ForbiddenKind / …
  serialize.rs  FEN + move-list (de)serialization
  rules/        win.rs, renju.rs, capture.rs, opening.rs, RuleSet presets
tests/          one integration suite per ruleset
examples/       demo.rs, bench.rs
benches/        engine.rs (Criterion)
```

Tests live in two places: fast unit tests in `#[cfg(test)]` modules next to the
code they cover, and black-box integration tests under `tests/` (one file per
ruleset, exercising only the public API). New behavior should come with tests in
whichever of those fits best.

## Expectations for a change

- **Public items carry a doc example.** The crate enforces
  `#![deny(missing_docs)]`, and the convention here is that every public item also
  has a `# Examples` block that runs under `cargo test --doc`. Match the existing
  style (use `?` with a trailing `# Ok::<(), ErrType>(())`, assert on outcomes).
- **Keep names consistent with their neighbors.** Match the surrounding code's
  idioms rather than introducing a new style.
- **Don't regress performance silently.** If you change a hot path (move legality,
  win/forbidden-move detection), sanity-check it with `cargo bench` or
  `cargo run --release --example bench` and mention the result.

## Commit and PR style

Small, focused commits with a clear message are easiest to review. Describe *why*
a change is made, not just *what*. If a PR is a work in progress or you just want
feedback on an approach, say so, that's fine.
