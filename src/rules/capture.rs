//! Pente-style custodial captures.
//!
//! When a player places a stone that brackets exactly two of the opponent's
//! stones against another of their own — the pattern `self · opp · opp · self`
//! along any of the eight directions — the two bracketed stones are captured
//! (removed). A pair placed *between* two enemy stones is not captured; only
//! the flanking move captures.

use crate::board::Board;
use crate::point::Point;
use crate::stone::Player;

/// The eight directions a capture can occur along.
const DIRS: [(i8, i8); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (-1, -1),
    (1, -1),
    (-1, 1),
];

/// Resolve captures triggered by `player` placing at `m` (already on the
/// board). Removes any captured stones and returns them (always in pairs).
pub(crate) fn resolve(board: &mut Board, m: Point, player: Player) -> Vec<Point> {
    let opp = player.opponent();
    let mut captured = Vec::new();

    for (dx, dy) in DIRS {
        let (Some(p1), Some(p2), Some(p3)) = (
            m.offset(dx, dy),
            m.offset(dx * 2, dy * 2),
            m.offset(dx * 3, dy * 3),
        ) else {
            continue;
        };
        if board.get(p1).is(opp) && board.get(p2).is(opp) && board.get(p3).is(player) {
            board.clear(p1);
            board.clear(p2);
            captured.push(p1);
            captured.push(p2);
        }
    }

    captured
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stone::Cell;

    #[test]
    fn flanking_move_captures_the_pair() {
        let mut b = Board::new(15);
        // White pair at (5,7),(6,7) flanked by Black at (4,7); Black plays (7,7).
        b.place(Player::Black, Point::new(4, 7));
        b.place(Player::White, Point::new(5, 7));
        b.place(Player::White, Point::new(6, 7));
        b.place(Player::Black, Point::new(7, 7));
        let taken = resolve(&mut b, Point::new(7, 7), Player::Black);
        assert_eq!(taken.len(), 2);
        assert!(b.is_empty(Point::new(5, 7)));
        assert!(b.is_empty(Point::new(6, 7)));
    }

    #[test]
    fn lone_stone_or_triple_is_not_captured() {
        // A single enemy stone (not a pair) is safe.
        let mut b = Board::new(15);
        b.place(Player::Black, Point::new(4, 7));
        b.place(Player::White, Point::new(5, 7));
        b.place(Player::Black, Point::new(6, 7));
        assert!(resolve(&mut b, Point::new(6, 7), Player::Black).is_empty());

        // Three enemy stones in a row are safe.
        let mut b = Board::new(15);
        b.place(Player::Black, Point::new(3, 7));
        for x in 4..7 {
            b.place(Player::White, Point::new(x, 7));
        }
        b.place(Player::Black, Point::new(7, 7));
        assert!(resolve(&mut b, Point::new(7, 7), Player::Black).is_empty());
    }

    #[test]
    fn placing_into_a_bracket_is_safe() {
        // Black plays between two White stones — Black is not captured.
        let mut b = Board::new(15);
        b.place(Player::White, Point::new(5, 7));
        b.place(Player::White, Point::new(8, 7));
        b.place(Player::Black, Point::new(6, 7));
        b.place(Player::Black, Point::new(7, 7));
        // White did not just move, so resolving Black's move captures nothing
        // of Black's own (resolve only ever removes the opponent of the mover).
        assert!(resolve(&mut b, Point::new(7, 7), Player::Black).is_empty());
        assert_eq!(b.get(Point::new(6, 7)), Cell::Stone(Player::Black));
    }
}
