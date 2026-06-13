//! Renju forbidden-move detection for Black.
//!
//! Black may not play a move that creates a double-three (two open threes), a
//! double-four (two fours), or an overline (six or more in a row) - *unless*
//! the same move also makes a five, which wins outright and is always allowed.
//! White is never restricted.
//!
//! Detection follows the standard recursive definition: an open three only
//! "counts" if the move that would turn it into a straight four is itself a
//! legal (non-forbidden) move for Black, so [`forbidden`] recurses into itself
//! when classifying threes. Recursion is bounded by [`MAX_DEPTH`].
//!
//! Fours and threes are counted per direction (a straight four therefore counts
//! once). Two fours or two threes sharing a single line, a vanishingly rare
//! shape in practice, and one the overline rule usually catches, are not
//! double-counted.

use crate::board::Board;
use crate::error::ForbiddenKind;
use crate::point::Point;
use crate::rules::win::AXES;
use crate::stone::{Cell, Player};

const BLACK: Player = Player::Black;

/// Maximum depth of the three-legality recursion. Real positions resolve in a
/// step or two; the cap only guards against pathological shapes.
const MAX_DEPTH: u8 = 6;

/// State of a cell along a line, relative to Black.
#[derive(PartialEq, Clone, Copy)]
enum Ls {
    Own,
    Opp,
    Empty,
    Edge,
}

/// Read the cell `k` steps from `from` along `dir`.
fn ls(board: &Board, from: Point, dir: (i8, i8), k: i8) -> Ls {
    match from.offset(dir.0 * k, dir.1 * k) {
        Some(q) if board.in_bounds(q) => match board.get(q) {
            Cell::Empty => Ls::Empty,
            Cell::Stone(BLACK) => Ls::Own,
            Cell::Stone(_) => Ls::Opp,
        },
        _ => Ls::Edge,
    }
}

/// The contiguous run of Black stones through `m` along `dir`, together with the
/// state of the cell just past each end. `m` must already hold a Black stone.
fn contiguous(board: &Board, m: Point, dir: (i8, i8)) -> (u32, Ls, Ls) {
    let mut f: i8 = 1;
    while ls(board, m, dir, f) == Ls::Own {
        f += 1;
    }
    let mut b: i8 = 1;
    while ls(board, m, dir, -b) == Ls::Own {
        b += 1;
    }
    let len = (f as u32 - 1) + (b as u32 - 1) + 1;
    (len, ls(board, m, dir, f), ls(board, m, dir, -b))
}

/// Whether the just-placed Black stone at `m` makes an exact five anywhere.
fn makes_five(board: &Board, m: Point) -> bool {
    AXES.iter().any(|&d| contiguous(board, m, d).0 == 5)
}

/// Whether the just-placed Black stone at `m` makes a run of six or more.
fn makes_overline(board: &Board, m: Point) -> bool {
    AXES.iter().any(|&d| contiguous(board, m, d).0 >= 6)
}

/// Whether Black has a four along `dir` through `m` (one move from an exact
/// five). `m` must already hold a Black stone. Temporarily places and removes a
/// completing stone, leaving `board` unchanged on return.
fn has_four(board: &mut Board, m: Point, dir: (i8, i8)) -> bool {
    // Examine every 5-cell window that contains `m`.
    for s in -4i8..=0 {
        let mut own = 0;
        let mut gap: Option<i8> = None;
        let mut dead = false;
        for j in 0..5i8 {
            match ls(board, m, dir, s + j) {
                Ls::Own => own += 1,
                Ls::Empty => gap = Some(s + j),
                _ => {
                    dead = true;
                    break;
                }
            }
        }
        if dead || own != 4 {
            continue;
        }
        // Exactly four own + one empty: filling the gap must give exactly five
        // (not an overline) for this to be a genuine four.
        let k = gap.unwrap();
        let e = m.offset(dir.0 * k, dir.1 * k).unwrap();
        board.place(BLACK, e);
        let five = contiguous(board, e, dir).0 == 5;
        board.clear(e);
        if five {
            return true;
        }
    }
    false
}

/// Whether Black has a *live* open three along `dir` through `m`: some empty
/// point on the line turns it into a straight four, and that point is itself a
/// legal move for Black. `m` must already hold a Black stone. `board` is left
/// unchanged on return.
fn has_open_three(board: &mut Board, m: Point, dir: (i8, i8), depth: u8) -> bool {
    for k in -4i8..=4 {
        if k == 0 || ls(board, m, dir, k) != Ls::Empty {
            continue;
        }
        let e = m.offset(dir.0 * k, dir.1 * k).unwrap();
        // A straight four through `m`: four contiguous with both ends open.
        board.place(BLACK, e);
        let (len, front, back) = contiguous(board, m, dir);
        board.clear(e);

        if len == 4 && front == Ls::Empty && back == Ls::Empty {
            // The completing move `e` must itself be legal for Black. `forbidden_at`
            // re-places and removes `e`, so we clear it first (above).
            if depth >= MAX_DEPTH || forbidden_at(board, e, depth + 1).is_none() {
                return true;
            }
        }
    }
    false
}

/// Classify a Black move at `m`, assuming `m` is already on `board`.
fn classify(board: &mut Board, m: Point, depth: u8) -> Option<ForbiddenKind> {
    if makes_five(board, m) {
        return None; // a five wins and overrides any forbidden shape
    }
    if makes_overline(board, m) {
        return Some(ForbiddenKind::Overline);
    }

    let mut fours = 0;
    for d in AXES {
        if has_four(board, m, d) {
            fours += 1;
        }
    }
    if fours >= 2 {
        return Some(ForbiddenKind::DoubleFour);
    }

    let mut threes = 0;
    for d in AXES {
        if !has_four(board, m, d) && has_open_three(board, m, d, depth) {
            threes += 1;
        }
    }
    if threes >= 2 {
        return Some(ForbiddenKind::DoubleThree);
    }

    None
}

/// Would Black playing at `m` be forbidden, given `board` (which does *not* yet
/// contain `m`)? Places `m`, classifies it, then removes it again so `board` is
/// unchanged on return.
fn forbidden_at(board: &mut Board, m: Point, depth: u8) -> Option<ForbiddenKind> {
    board.place(BLACK, m);
    let verdict = classify(board, m, depth);
    board.clear(m);
    verdict
}

/// Maximum distance along an axis at which an existing Black stone can contribute
/// to a shape forbidden by a move at `m`. A four or an open three may carry a
/// single gap immediately next to `m`, leaving the nearest *real* Black stone two
/// cells away; a five or an overline runs contiguously through `m`, so its stones
/// are adjacent. No forbidden shape can be built without a Black stone this close,
/// so a move with none in range is always allowed.
const FORBIDDEN_REACH: i8 = 2;

/// Whether any Black stone sits within [`FORBIDDEN_REACH`] of `m` along some axis,
/// i.e. close enough to take part in a forbidden shape. A move that fails this
/// cheap check cannot be forbidden, so [`forbidden`] can skip classifying it. The
/// scan is conservative, it reads through gaps and opponent stones rather than
/// stopping, so it may over-report (costing a redundant classification) but can
/// never miss a genuinely forbidden move.
fn black_in_range(board: &Board, m: Point) -> bool {
    for &(dx, dy) in AXES.iter() {
        for sign in [1i8, -1] {
            for k in 1..=FORBIDDEN_REACH {
                match m.offset(dx * sign * k, dy * sign * k) {
                    Some(q) if board.get(q).is(BLACK) => return true,
                    Some(_) => {}
                    None => break, // ran off the edge along this direction
                }
            }
        }
    }
    false
}

/// Whether Black playing at `m` is a Renju forbidden move. `board` is the
/// current position (without `m`).
///
/// A cheap range check rejects the common case, a point with no Black stone
/// close enough to form any shape, before the heavier classification. Otherwise
/// the board is copied once into a scratch position, after which every
/// hypothetical stone (including the recursive three-resolution) is tested by
/// make/unmake on that single board, no per-candidate copies.
pub(crate) fn forbidden(board: &Board, m: Point) -> Option<ForbiddenKind> {
    if !black_in_range(board, m) {
        return None;
    }
    let mut scratch = board.clone();
    forbidden_at(&mut scratch, m, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Place Black stones at the given algebraic-style `(x, y)` points.
    fn black_board(stones: &[(u8, u8)]) -> Board {
        let mut b = Board::new(15);
        for &(x, y) in stones {
            b.place(Player::Black, Point::new(x, y));
        }
        b
    }

    #[test]
    fn overline_is_forbidden() {
        // Black at x = 4,5,6,7,9 on row 7; playing x = 8 makes six in a row.
        let b = black_board(&[(4, 7), (5, 7), (6, 7), (7, 7), (9, 7)]);
        assert_eq!(
            forbidden(&b, Point::new(8, 7)),
            Some(ForbiddenKind::Overline)
        );
    }

    #[test]
    fn five_is_allowed_even_if_it_looks_forbidden() {
        // Exactly five, always legal regardless of other shapes.
        let b = black_board(&[(4, 7), (5, 7), (6, 7), (7, 7)]);
        assert_eq!(forbidden(&b, Point::new(8, 7)), None);
        assert_eq!(forbidden(&b, Point::new(3, 7)), None);
    }

    #[test]
    fn double_four_is_forbidden() {
        // Two gapped fours crossing at (7,7): playing it yields X X X _ X both
        // horizontally and vertically (a four each), without making a five.
        let b = black_board(&[
            (3, 7),
            (4, 7),
            (5, 7), // horizontal: X X X _ (7,7) with gap at (6,7)
            (7, 3),
            (7, 4),
            (7, 5), // vertical:   X X X _ (7,7) with gap at (7,6)
        ]);
        assert_eq!(
            forbidden(&b, Point::new(7, 7)),
            Some(ForbiddenKind::DoubleFour)
        );
    }

    #[test]
    fn double_three_is_forbidden() {
        // Two open threes meeting at (7,7): horizontal _ X X _ and vertical
        // _ X X _, completed into a 3-3 fork by playing (7,7).
        let b = black_board(&[(5, 7), (6, 7), (7, 5), (7, 6)]);
        assert_eq!(
            forbidden(&b, Point::new(7, 7)),
            Some(ForbiddenKind::DoubleThree)
        );
    }

    #[test]
    fn single_three_is_allowed() {
        let b = black_board(&[(5, 7), (6, 7)]);
        assert_eq!(forbidden(&b, Point::new(7, 7)), None);
    }

    #[test]
    fn single_four_is_allowed() {
        let b = black_board(&[(3, 7), (4, 7), (5, 7), (6, 7)]);
        // Extending to a straight/open four is fine (it is a winning threat).
        assert_eq!(forbidden(&b, Point::new(2, 7)), None);
    }

    #[test]
    fn overline_with_move_at_run_end_is_forbidden() {
        // The move caps a six-in-a-row from its end; the `black_in_range` pre-check
        // must still admit it for classification (the run is adjacent to `m`).
        let b = black_board(&[(3, 7), (4, 7), (5, 7), (6, 7), (7, 7)]);
        assert_eq!(
            forbidden(&b, Point::new(8, 7)),
            Some(ForbiddenKind::Overline)
        );
    }

    #[test]
    fn move_far_from_all_stones_is_allowed() {
        // The `black_in_range` fast-path: with no Black stone anywhere near `m`,
        // no forbidden shape is possible and classification is skipped.
        let b = black_board(&[(2, 2), (2, 3)]);
        assert_eq!(forbidden(&b, Point::new(12, 12)), None);
    }
}
