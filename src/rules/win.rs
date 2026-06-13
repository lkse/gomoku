//! Win detection by scanning the four axes through the last placed stone.
//!
//! Because only the most recent stone can complete a new line, a move can be
//! tested in O(1): walk outward along each axis counting same-color stones.

use crate::board::Board;
use crate::point::Point;
use crate::rules::{Overline, RuleSet};
use crate::stone::{Cell, Player};

/// The four line orientations: horizontal, vertical, and the two diagonals.
/// Only one half of each direction pair is listed; [`run_through`] walks both
/// ways from the stone.
pub(crate) const AXES: [(i8, i8); 4] = [(1, 0), (0, 1), (1, 1), (-1, 1)];

/// The contiguous same-color run through a point along one axis.
pub(crate) struct LineRun {
    /// Length of the run, including the stone at the origin point.
    pub len: u8,
    /// How many of the two ends are bounded by an opponent stone (0, 1, or 2).
    /// The board edge does not count as a block.
    pub blocked_ends: u8,
}

/// Measure the run of `player`'s stones through `p` along `axis`.
pub(crate) fn run_through(board: &Board, p: Point, player: Player, axis: (i8, i8)) -> LineRun {
    let (dx, dy) = axis;
    let mut len = 1u8;
    let mut blocked_ends = 0u8;

    for sign in [1i8, -1i8] {
        let (sdx, sdy) = (dx * sign, dy * sign);
        let mut step: i8 = 1;
        loop {
            // Read each cell once; off-board reads as empty (not a block).
            match p.offset(sdx * step, sdy * step).map(|q| board.get(q)) {
                Some(Cell::Stone(c)) if c == player => {
                    len += 1;
                    step += 1;
                }
                Some(Cell::Stone(_)) => {
                    blocked_ends += 1; // an opponent stone bounds this end
                    break;
                }
                Some(Cell::Empty) | None => break,
            }
        }
    }

    LineRun { len, blocked_ends }
}

/// Whether placing `player` at `p` (already on the board) wins under `rules`.
pub(crate) fn is_win(board: &Board, p: Point, player: Player, rules: &RuleSet) -> bool {
    // Renju restricts only Black: Black wins on exactly five, while White may
    // also win with an overline. Other variants treat both colors the same.
    let overline = if rules.forbidden_black {
        match player {
            Player::Black => Overline::NoWin,
            Player::White => Overline::Win,
        }
    } else {
        rules.overline
    };
    for axis in AXES {
        let run = run_through(board, p, player, axis);
        let won = match overline {
            Overline::NoWin => run.len == rules.win_length,
            Overline::Win => {
                run.len >= rules.win_length
                    && !(rules.caro_block_both_ends && run.blocked_ends >= 2)
            }
        };
        if won {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::point::Point;

    fn place_row(board: &mut Board, player: Player, start: Point, axis: (i8, i8), n: i8) {
        for i in 0..n {
            let q = start.offset(axis.0 * i, axis.1 * i).unwrap();
            board.place(player, q);
        }
    }

    #[test]
    fn freestyle_five_and_overline_win() {
        let rules = RuleSet::freestyle();
        let mut b = Board::new(15);
        place_row(&mut b, Player::Black, Point::new(2, 2), (1, 0), 5);
        assert!(is_win(&b, Point::new(4, 2), Player::Black, &rules));

        let mut b = Board::new(15);
        place_row(&mut b, Player::Black, Point::new(2, 2), (1, 0), 6);
        assert!(is_win(&b, Point::new(4, 2), Player::Black, &rules));
    }

    #[test]
    fn standard_rejects_overline() {
        let rules = RuleSet::standard();
        let mut b = Board::new(15);
        place_row(&mut b, Player::Black, Point::new(2, 2), (1, 0), 6);
        // The completing stone sits inside a six -> no win under Standard.
        assert!(!is_win(&b, Point::new(4, 2), Player::Black, &rules));

        let mut b = Board::new(15);
        place_row(&mut b, Player::Black, Point::new(2, 2), (1, 0), 5);
        assert!(is_win(&b, Point::new(4, 2), Player::Black, &rules));
    }

    #[test]
    fn diagonal_win() {
        let rules = RuleSet::standard();
        let mut b = Board::new(15);
        place_row(&mut b, Player::White, Point::new(1, 1), (1, 1), 5);
        assert!(is_win(&b, Point::new(3, 3), Player::White, &rules));
    }
}
