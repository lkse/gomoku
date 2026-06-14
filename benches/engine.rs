//! Rigorous per-move timings via Criterion.
//!
//! Run with `cargo bench`. Each benchmark measures one `play(point)` plus the
//! matching `undo()` from a fixed mid-game position, so the reported time is the
//! per-move cost of legality + win detection (plus Renju forbidden-move checks
//! or Pente capture scanning where applicable). A second group measures full
//! legal-move generation across the board.

use codspeed_criterion_compat::{black_box, criterion_group, criterion_main, Criterion};
use gomoku::{Game, Point, RuleSet, Status};

/// A named rule variant and the preset that builds it.
type Mode = (&'static str, fn() -> RuleSet);

const MODES: [Mode; 6] = [
    ("freestyle", RuleSet::freestyle),
    ("standard", RuleSet::standard),
    ("renju", RuleSet::renju),
    ("caro", RuleSet::caro),
    ("omok", RuleSet::omok),
    ("pente", RuleSet::pente),
];

/// Deterministic xorshift, matching the example's PRNG.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

/// Build a reproducible mid-game position with roughly `target` stones placed,
/// stopping while the game is still in progress, and return a legal point for
/// the side to move to use as the benchmarked move.
fn mid_game(rules: RuleSet, size: u8, target: usize) -> (Game, Point) {
    let mut g = Game::new(rules.with_board_size(size));
    let mut rng = Rng(0xDEADBEEFCAFE1234);
    let n = (size as usize) * (size as usize);

    while g.move_count() < target && g.status() == Status::InProgress {
        let start = (rng.next() % n as u64) as usize;
        for k in 0..n {
            let idx = (start + k) % n;
            let p = Point::new((idx % size as usize) as u8, (idx / size as usize) as u8);
            if g.play(p).is_ok() {
                break;
            }
        }
    }

    // Find any legal point for the benchmarked move.
    let n = (size as usize) * (size as usize);
    let bench_point = (0..n)
        .map(|i| Point::new((i % size as usize) as u8, (i / size as usize) as u8))
        .find(|&p| g.is_legal(p))
        .expect("a legal move should exist in the mid-game position");

    (g, bench_point)
}

fn bench_play_undo(c: &mut Criterion) {
    let mut group = c.benchmark_group("play_undo_15x15");
    for (name, make) in MODES {
        let (mut game, p) = mid_game(make(), 15, 40);
        group.bench_function(name, |b| {
            b.iter(|| {
                let _ = game.play(black_box(p));
                game.undo();
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("play_undo_19x19");
    for (name, make) in MODES {
        let (mut game, p) = mid_game(make(), 19, 60);
        group.bench_function(name, |b| {
            b.iter(|| {
                let _ = game.play(black_box(p));
                game.undo();
            });
        });
    }
    group.finish();
}

fn bench_legal_moves(c: &mut Criterion) {
    let mut group = c.benchmark_group("legal_moves_15x15");
    for (name, make) in MODES {
        let (game, _) = mid_game(make(), 15, 40);
        group.bench_function(name, |b| {
            b.iter(|| black_box(game.legal_moves()));
        });
    }
    group.finish();
}

criterion_group!(benches, bench_play_undo, bench_legal_moves);
criterion_main!(benches);
