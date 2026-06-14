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
enum LineCell {
    /// Holds a Black stone (the color whose shapes are being classified).
    Own,
    /// Holds a White stone.
    Opp,
    /// Unoccupied and on the board.
    Empty,
    /// Off the board.
    Edge,
}

/// Read the cell `step` cells from `from` along `dir`.
fn cell_at(board: &Board, from: Point, dir: (i8, i8), step: i8) -> LineCell {
    match from.offset(dir.0 * step, dir.1 * step) {
        Some(q) if board.in_bounds(q) => match board.get(q) {
            Cell::Empty => LineCell::Empty,
            Cell::Stone(BLACK) => LineCell::Own,
            Cell::Stone(_) => LineCell::Opp,
        },
        _ => LineCell::Edge,
    }
}

/// The contiguous run of Black stones through `m` along `dir`, together with the
/// state of the cell just past each end. `m` must already hold a Black stone.
fn contiguous(board: &Board, m: Point, dir: (i8, i8)) -> (u32, LineCell, LineCell) {
    let mut forward: i8 = 1;
    while cell_at(board, m, dir, forward) == LineCell::Own {
        forward += 1;
    }
    let mut backward: i8 = 1;
    while cell_at(board, m, dir, -backward) == LineCell::Own {
        backward += 1;
    }
    let len = (forward as u32 - 1) + (backward as u32 - 1) + 1;
    (
        len,
        cell_at(board, m, dir, forward),
        cell_at(board, m, dir, -backward),
    )
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
            match cell_at(board, m, dir, s + j) {
                LineCell::Own => own += 1,
                LineCell::Empty => gap = Some(s + j),
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
        let fill_offset = gap.unwrap();
        let fill = m.offset(dir.0 * fill_offset, dir.1 * fill_offset).unwrap();
        board.place(BLACK, fill);
        let five = contiguous(board, fill, dir).0 == 5;
        board.clear(fill);
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
        if k == 0 || cell_at(board, m, dir, k) != LineCell::Empty {
            continue;
        }
        let fill = m.offset(dir.0 * k, dir.1 * k).unwrap();
        // A straight four through `m`: four contiguous with both ends open.
        board.place(BLACK, fill);
        let (len, front, back) = contiguous(board, m, dir);
        board.clear(fill);

        if len == 4 && front == LineCell::Empty && back == LineCell::Empty {
            // The completing move `fill` must itself be legal for Black.
            // `forbidden_at` re-places and removes `fill`, so we clear it first.
            if depth >= MAX_DEPTH || forbidden_at(board, fill, depth + 1).is_none() {
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
///
/// Kept out-of-line: this is only reached on the Renju branch of `move_playable`,
/// and inlining its bulk there would bloat that shared hot path and slow move
/// generation for every variant (a `lto` + `codegen-units = 1` layout effect).
#[inline(never)]
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
