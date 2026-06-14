//! The board: a pair of per-color bitboards plus access and rendering helpers.

use crate::point::{Point, MAX_SIZE};
use crate::stone::{Cell, Player};
use std::fmt;

/// Number of 64-bit words needed to cover `MAX_SIZE * MAX_SIZE` (= 361) cells.
const WORDS: usize = 6;

/// A gomoku board of configurable square size (`5..=19`).
///
/// Occupancy is stored as two bitboards, one per color, so that membership
/// tests and whole-board scans are a handful of word operations regardless of
/// size. Cells are indexed `y * size + x`.
///
/// Most callers reach the board through [`Game::board`](crate::Game::board)
/// rather than mutating one directly, but the placement API is public so the
/// board can be used as a standalone bitboard.
///
/// # Examples
///
/// ```
/// use gomoku::{Board, Cell, Player, Point};
///
/// let mut board = Board::new(15);
/// board.place(Player::Black, Point::new(7, 7));
///
/// assert_eq!(board.get(Point::new(7, 7)), Cell::Stone(Player::Black));
/// assert!(board.is_empty(Point::new(0, 0)));
/// assert_eq!(board.stone_count(), 1);
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    size: u8,
    black: [u64; WORDS],
    white: [u64; WORDS],
}

impl Board {
    /// Create an empty `size x size` board.
    ///
    /// # Panics
    ///
    /// Panics unless `5 <= size <= 19`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Board;
    ///
    /// let board = Board::new(19);
    /// assert_eq!(board.size(), 19);
    /// assert_eq!(board.stone_count(), 0);
    /// ```
    #[must_use]
    pub fn new(size: u8) -> Board {
        assert!(
            (5..=MAX_SIZE).contains(&size),
            "board size must be in 5..=19, got {size}"
        );
        Board {
            size,
            black: [0; WORDS],
            white: [0; WORDS],
        }
    }

    /// The edge length of the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Board;
    ///
    /// assert_eq!(Board::new(15).size(), 15);
    /// ```
    #[inline]
    pub const fn size(&self) -> u8 {
        self.size
    }

    /// Whether the point lies on the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Point};
    ///
    /// let board = Board::new(15);
    /// assert!(board.in_bounds(Point::new(14, 14)));
    /// assert!(!board.in_bounds(Point::new(15, 0))); // column 15 is off a 15-wide board
    /// ```
    #[inline]
    pub const fn in_bounds(&self, p: Point) -> bool {
        p.x < self.size && p.y < self.size
    }

    /// Linear bit index of a point. Caller must ensure the point is in bounds.
    #[inline]
    const fn index(&self, p: Point) -> usize {
        p.y as usize * self.size as usize + p.x as usize
    }

    /// Contents of a cell. Off-board points read as [`Cell::Empty`].
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Cell, Player, Point};
    ///
    /// let mut board = Board::new(15);
    /// board.place(Player::White, Point::new(3, 4));
    ///
    /// assert_eq!(board.get(Point::new(3, 4)), Cell::Stone(Player::White));
    /// assert_eq!(board.get(Point::new(0, 0)), Cell::Empty);
    /// assert_eq!(board.get(Point::new(99, 99)), Cell::Empty); // off-board reads as empty
    /// ```
    #[inline]
    pub fn get(&self, p: Point) -> Cell {
        if !self.in_bounds(p) {
            return Cell::Empty;
        }
        let i = self.index(p);
        let (w, bit) = (i / 64, 1u64 << (i % 64));
        if self.black[w] & bit != 0 {
            Cell::Stone(Player::Black)
        } else if self.white[w] & bit != 0 {
            Cell::Stone(Player::White)
        } else {
            Cell::Empty
        }
    }

    /// Whether the in-bounds point is unoccupied.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Player, Point};
    ///
    /// let mut board = Board::new(15);
    /// assert!(board.is_empty(Point::new(7, 7)));
    /// board.place(Player::Black, Point::new(7, 7));
    /// assert!(!board.is_empty(Point::new(7, 7)));
    /// ```
    #[inline]
    pub fn is_empty(&self, p: Point) -> bool {
        matches!(self.get(p), Cell::Empty)
    }

    /// Place a stone, overwriting whatever was there.
    ///
    /// # Panics
    /// Panics if the point is off-board.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Cell, Player, Point};
    ///
    /// let mut board = Board::new(15);
    /// let p = Point::new(7, 7);
    ///
    /// board.place(Player::Black, p);
    /// assert_eq!(board.get(p), Cell::Stone(Player::Black));
    ///
    /// // Placing the other color overwrites in place.
    /// board.place(Player::White, p);
    /// assert_eq!(board.get(p), Cell::Stone(Player::White));
    /// assert_eq!(board.stone_count(), 1);
    /// ```
    #[inline]
    pub fn place(&mut self, player: Player, at: Point) {
        assert!(self.in_bounds(at), "place out of bounds: {at}");
        let i = self.index(at);
        let (w, bit) = (i / 64, 1u64 << (i % 64));
        match player {
            Player::Black => {
                self.black[w] |= bit;
                self.white[w] &= !bit;
            }
            Player::White => {
                self.white[w] |= bit;
                self.black[w] &= !bit;
            }
        }
    }

    /// Remove any stone at the point, leaving it empty. A no-op off-board.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Player, Point};
    ///
    /// let mut board = Board::new(15);
    /// let p = Point::new(7, 7);
    /// board.place(Player::Black, p);
    ///
    /// board.clear(p);
    /// assert!(board.is_empty(p));
    /// ```
    #[inline]
    pub fn clear(&mut self, at: Point) {
        if !self.in_bounds(at) {
            return;
        }
        let i = self.index(at);
        let (w, bit) = (i / 64, 1u64 << (i % 64));
        self.black[w] &= !bit;
        self.white[w] &= !bit;
    }

    /// Total number of stones on the board.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Player, Point};
    ///
    /// let mut board = Board::new(15);
    /// board.place(Player::Black, Point::new(7, 7));
    /// board.place(Player::White, Point::new(7, 8));
    /// assert_eq!(board.stone_count(), 2);
    /// ```
    #[inline]
    pub fn stone_count(&self) -> u32 {
        (0..WORDS)
            .map(|w| self.black[w].count_ones() + self.white[w].count_ones())
            .sum()
    }

    /// Whether every intersection is occupied (used for draw detection).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Player};
    ///
    /// // Fill a tiny 5×5 board completely.
    /// let mut board = Board::new(5);
    /// for p in board.points().collect::<Vec<_>>() {
    ///     board.place(Player::Black, p);
    /// }
    /// assert!(board.is_full());
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        self.stone_count() == self.size as u32 * self.size as u32
    }

    /// Iterate over every point on the board, row by row from the bottom-left.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Board;
    ///
    /// let board = Board::new(15);
    /// assert_eq!(board.points().count(), 15 * 15);
    /// ```
    pub fn points(&self) -> impl Iterator<Item = Point> + '_ {
        let size = self.size;
        (0..size).flat_map(move |y| (0..size).map(move |x| Point::new(x, y)))
    }

    /// The center intersection (defined for odd sizes; rounds down otherwise).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Point};
    ///
    /// assert_eq!(Board::new(15).center(), Point::new(7, 7));
    /// ```
    #[inline]
    pub const fn center(&self) -> Point {
        Point::new(self.size / 2, self.size / 2)
    }
}

/// Renders the board as text with row numbers down the left and column letters
/// along the bottom, origin at the bottom-left. Black stones show as `X`, White
/// as `O`, and empty intersections as `.`.
///
/// # Examples
///
/// ```
/// use gomoku::{Board, Player, Point};
///
/// let mut board = Board::new(15);
/// board.place(Player::Black, Point::new(7, 7));
///
/// let text = board.to_string();
/// assert!(text.contains('X'));     // the black stone we placed
/// assert!(text.contains("a b c")); // column labels along the bottom edge
/// ```
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Rows are printed top (highest y) to bottom so the origin sits at the
        // bottom-left, matching the algebraic coordinate convention.
        for y in (0..self.size).rev() {
            write!(f, "{:>2} ", y as u32 + 1)?;
            for x in 0..self.size {
                write!(f, "{} ", self.get(Point::new(x, y)).glyph())?;
            }
            writeln!(f)?;
        }
        write!(f, "   ")?;
        for x in 0..self.size {
            write!(f, "{} ", (b'a' + x) as char)?;
        }
        Ok(())
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Board({}x{})\n{}", self.size, self.size, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_get_clear() {
        let mut b = Board::new(15);
        let p = Point::new(7, 7);
        assert!(b.is_empty(p));
        b.place(Player::Black, p);
        assert_eq!(b.get(p), Cell::Stone(Player::Black));
        assert_eq!(b.stone_count(), 1);
        // Overwrite with the other color.
        b.place(Player::White, p);
        assert_eq!(b.get(p), Cell::Stone(Player::White));
        assert_eq!(b.stone_count(), 1);
        b.clear(p);
        assert!(b.is_empty(p));
        assert_eq!(b.stone_count(), 0);
    }

    #[test]
    fn high_index_uses_upper_words() {
        // Corner of a 19x19 board lives at bit 360, in word 5.
        let mut b = Board::new(19);
        let corner = Point::new(18, 18);
        b.place(Player::White, corner);
        assert_eq!(b.get(corner), Cell::Stone(Player::White));
        assert!(b.is_empty(Point::new(0, 0)));
    }

    #[test]
    fn bounds_and_center() {
        let b = Board::new(15);
        assert!(b.in_bounds(Point::new(14, 14)));
        assert!(!b.in_bounds(Point::new(15, 0)));
        assert_eq!(b.center(), Point::new(7, 7));
        assert!(b.get(Point::new(99, 99)).is_empty());
    }

    #[test]
    fn points_cover_whole_board() {
        let b = Board::new(15);
        assert_eq!(b.points().count(), 225);
    }

    #[test]
    #[should_panic]
    fn rejects_tiny_board() {
        let _ = Board::new(4);
    }
}
