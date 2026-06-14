//! Board coordinates.

use std::fmt;

/// The largest board edge this engine supports.
///
/// # Examples
///
/// ```
/// use gomoku::MAX_SIZE;
///
/// assert_eq!(MAX_SIZE, 19);
/// ```
pub const MAX_SIZE: u8 = 19;

/// A board intersection, addressed by zero-based column (`x`) and row (`y`).
///
/// The origin `(0, 0)` is the bottom-left corner, matching the conventional
/// gomoku/renju coordinate system where columns run `A..` left-to-right and
/// rows run `1..` bottom-to-top.
///
/// # Examples
///
/// ```
/// use gomoku::Point;
///
/// // The center of a 15×15 board, `h8` in algebraic notation.
/// let p = Point::new(7, 7);
/// assert_eq!((p.x, p.y), (7, 7));
/// assert_eq!(p.to_string(), "h8");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    /// Zero-based column index, increasing left to right.
    pub x: u8,
    /// Zero-based row index, increasing bottom to top.
    pub y: u8,
}

impl Point {
    /// Construct a point from its column (`x`) and row (`y`) indices.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Point;
    ///
    /// let p = Point::new(3, 5);
    /// assert_eq!(p.x, 3);
    /// assert_eq!(p.y, 5);
    /// ```
    #[inline]
    pub const fn new(x: u8, y: u8) -> Point {
        Point { x, y }
    }

    /// Offset this point by `(dx, dy)`, returning `None` if it would leave the
    /// coordinate space (including underflow below zero).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Point;
    ///
    /// let p = Point::new(0, 0);
    /// assert_eq!(p.offset(1, 2), Some(Point::new(1, 2)));
    /// // Stepping below the bottom-left corner leaves the coordinate space.
    /// assert_eq!(p.offset(-1, 0), None);
    /// ```
    #[inline]
    pub fn offset(self, dx: i8, dy: i8) -> Option<Point> {
        let x = (self.x as i16) + dx as i16;
        let y = (self.y as i16) + dy as i16;
        if (0..MAX_SIZE as i16).contains(&x) && (0..MAX_SIZE as i16).contains(&y) {
            Some(Point::new(x as u8, y as u8))
        } else {
            None
        }
    }

    /// Parse algebraic coordinates such as `"h8"` or `"A1"`.
    ///
    /// Columns are letters `a..` (case-insensitive); rows are 1-based numbers.
    /// This is the inverse of the [`Display`](std::fmt::Display) formatting, so
    /// any point round-trips through its string form.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Point;
    ///
    /// assert_eq!(Point::parse("h8"), Some(Point::new(7, 7)));
    /// assert_eq!(Point::parse("A1"), Some(Point::new(0, 0))); // letters are case-insensitive
    /// assert_eq!(Point::parse("a0"), None);                   // rows start at 1
    /// assert_eq!(Point::parse(""), None);
    ///
    /// // Round-trips with `to_string`.
    /// let p = Point::new(11, 2);
    /// assert_eq!(Point::parse(&p.to_string()), Some(p));
    /// ```
    pub fn parse(s: &str) -> Option<Point> {
        let s = s.trim();
        let mut chars = s.chars();
        let col = chars.next()?.to_ascii_lowercase();
        if !col.is_ascii_lowercase() {
            return None;
        }
        let x = (col as u8) - b'a';
        let row: u32 = chars.as_str().parse().ok()?;
        if row == 0 {
            return None;
        }
        let y = u8::try_from(row - 1).ok()?;
        Some(Point::new(x, y))
    }
}

/// Formats a point in algebraic notation, e.g. `"h8"` - the inverse of
/// [`Point::parse`].
///
/// # Examples
///
/// ```
/// use gomoku::Point;
///
/// assert_eq!(Point::new(7, 7).to_string(), "h8");
/// assert_eq!(Point::new(0, 0).to_string(), "a1");
/// ```
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", (b'a' + self.x) as char, self.y as u32 + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset_stays_in_bounds() {
        let p = Point::new(0, 0);
        assert_eq!(p.offset(-1, 0), None);
        assert_eq!(p.offset(0, -1), None);
        assert_eq!(p.offset(1, 1), Some(Point::new(1, 1)));
        assert_eq!(Point::new(18, 18).offset(1, 0), None);
    }

    #[test]
    fn algebraic_roundtrip() {
        assert_eq!(Point::parse("a1"), Some(Point::new(0, 0)));
        assert_eq!(Point::parse("H8"), Some(Point::new(7, 7)));
        assert_eq!(Point::new(7, 7).to_string(), "h8");
        assert_eq!(Point::parse("a0"), None);
        assert_eq!(Point::parse(""), None);
    }
}
