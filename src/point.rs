//! Board coordinates.

use std::fmt;

/// The largest board edge this engine supports.
pub const MAX_SIZE: u8 = 19;

/// A board intersection, addressed by zero-based column (`x`) and row (`y`).
///
/// The origin `(0, 0)` is the bottom-left corner, matching the conventional
/// gomoku/renju coordinate system where columns run `A..` left-to-right and
/// rows run `1..` bottom-to-top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Point {
    /// Zero-based column index, increasing left to right.
    pub x: u8,
    /// Zero-based row index, increasing bottom to top.
    pub y: u8,
}

impl Point {
    /// Construct a point from its column (`x`) and row (`y`) indices.
    #[inline]
    pub const fn new(x: u8, y: u8) -> Point {
        Point { x, y }
    }

    /// Offset this point by `(dx, dy)`, returning `None` if it would leave the
    /// coordinate space (including underflow below zero).
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
