//! A dependency-free self-play benchmark and outcome survey.
//! Run with `cargo run --release --example bench`.
//!
//! For every rule variant, at both 15×15 and 19×19, it plays many pseudo-random
//! games to completion and reports throughput, average per-move time, and the
//! distribution of outcomes. Progress is printed as it goes.
//!
//! For rigorous per-move timing distributions (warmup, outliers, confidence
//! intervals) see the Criterion suite: `cargo bench`.
//!
//! Only the variants driven entirely by `play()` are covered. The opening
//! protocols (Swap/Swap2/Yamaguchi) need interactive decision calls and so are
//! not part of a random self-play loop.

use gomoku::{Game, Player, Point, RuleSet, Status};
use std::io::{self, Write};
use std::time::Instant;

/// Games to play per (variant, size) cell.
const GAMES: u64 = 100_000;
/// Board sizes to sweep.
const SIZES: [u8; 2] = [15, 19];

/// A named rule variant and the preset that builds it.
type Mode = (&'static str, fn() -> RuleSet);

/// A tiny xorshift PRNG so the example needs no `rand` dependency.
struct Rng(u64);
impl Rng {
    #[inline]
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// How a self-play game ended.
enum Outcome {
    BlackWin,
    WhiteWin,
    Draw,
    /// Black had no legal move (every empty point forbidden) — a Renju loss.
    BlackForbiddenLoss,
}

/// Tallies for one (variant, size) cell.
#[derive(Default)]
struct Stats {
    moves: u64,
    black_wins: u64,
    white_wins: u64,
    draws: u64,
    renju_losses: u64,
}

/// Play one game, choosing each move by scanning from a random offset for the
/// first point the current player may legally take. Returns the outcome and the
/// number of stones placed.
fn play_one(rules: RuleSet, rng: &mut Rng) -> (Outcome, u64) {
    let mut g = Game::new(rules);
    let size = g.board().size() as usize;
    let n = size * size;
    let mut moves = 0u64;

    loop {
        match g.status() {
            Status::Win(Player::Black) => return (Outcome::BlackWin, moves),
            Status::Win(Player::White) => return (Outcome::WhiteWin, moves),
            Status::Draw => return (Outcome::Draw, moves),
            Status::InProgress => {}
        }

        let player = g.to_move();
        let start = (rng.next() % n as u64) as usize;
        let mut placed = false;
        for k in 0..n {
            let idx = (start + k) % n;
            let p = Point::new((idx % size) as u8, (idx / size) as u8);
            // play() mutates only on success, so a rejected (occupied or
            // forbidden) point simply lets us try the next one.
            if g.play(p).is_ok() {
                moves += 1;
                placed = true;
                break;
            }
        }

        if !placed {
            // No legal move exists for `player`. Under Renju that is Black
            // losing by the forbidden-move rules; otherwise it is a full board.
            return if rules.forbidden_black && player == Player::Black {
                (Outcome::BlackForbiddenLoss, moves)
            } else {
                (Outcome::Draw, moves)
            };
        }
    }
}

fn main() {
    let modes: [Mode; 6] = [
        ("freestyle", RuleSet::freestyle),
        ("standard", RuleSet::standard),
        ("renju", RuleSet::renju),
        ("caro", RuleSet::caro),
        ("omok", RuleSet::omok),
        ("pente", RuleSet::pente),
    ];

    let mut rng = Rng(0x9E3779B97F4A7C15);
    println!("{GAMES} games per variant per size\n");
    println!(
        "{:<10} {:>5} {:>13} {:>8} {:>9} {:>8} {:>8} {:>7} {:>9}",
        "variant", "board", "moves", "time", "ns/move", "B-win", "W-win", "draw", "renju-L"
    );
    println!("{}", "-".repeat(86));

    let chunk = (GAMES / 20).max(1);
    for (name, make) in modes {
        for size in SIZES {
            let rules = make().with_board_size(size);
            let label = format!("{name} {size}x{size}");

            let start = Instant::now();
            let mut s = Stats::default();
            for i in 0..GAMES {
                let (outcome, moves) = play_one(rules, &mut rng);
                s.moves += moves;
                match outcome {
                    Outcome::BlackWin => s.black_wins += 1,
                    Outcome::WhiteWin => s.white_wins += 1,
                    Outcome::Draw => s.draws += 1,
                    Outcome::BlackForbiddenLoss => s.renju_losses += 1,
                }
                if (i + 1) % chunk == 0 {
                    print!("\r  {label}: {}%   ", (i + 1) * 100 / GAMES);
                    let _ = io::stdout().flush();
                }
            }
            let secs = start.elapsed().as_secs_f64();
            let ns_per_move = secs * 1e9 / s.moves as f64;

            // Overwrite the progress line with the finished row.
            print!("\r");
            println!(
                "{:<10} {:>5} {:>13} {:>7.2}s {:>8.1} {:>8} {:>8} {:>7} {:>9}    ",
                name,
                format!("{size}x{size}"),
                s.moves,
                secs,
                ns_per_move,
                s.black_wins,
                s.white_wins,
                s.draws,
                s.renju_losses,
            );
        }
    }

    println!("\nthroughput summary (moves/s):");
    println!("  see ns/move column; e.g. 60 ns/move ≈ 16.7 M moves/s");
}
