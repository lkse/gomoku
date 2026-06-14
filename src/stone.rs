//! The two players and the contents of a board cell.

/// One of the two players. Black always moves first.
///
/// # Examples
///
/// ```
/// use gomoku::{Game, Player, RuleSet};
///
/// // Black is always the side to move in a fresh game.
/// let game = Game::new(RuleSet::standard());
/// assert_eq!(game.to_move(), Player::Black);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    /// The first player. Renders as `X` and always moves first.
    Black,
    /// The second player. Renders as `O`.
    White,
}

impl Player {
    /// The other player.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Player;
    ///
    /// assert_eq!(Player::Black.opponent(), Player::White);
    /// assert_eq!(Player::White.opponent(), Player::Black);
    /// ```
    #[inline]
    pub const fn opponent(self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    /// The single character used to render this player's stones (`X` / `O`).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::Player;
    ///
    /// assert_eq!(Player::Black.glyph(), 'X');
    /// assert_eq!(Player::White.glyph(), 'O');
    /// ```
    #[inline]
    pub const fn glyph(self) -> char {
        match self {
            Player::Black => 'X',
            Player::White => 'O',
        }
    }
}

/// The contents of a single intersection on the board, as returned by
/// [`Board::get`](crate::Board::get).
///
/// # Examples
///
/// ```
/// use gomoku::{Cell, Player};
///
/// let occupied = Cell::Stone(Player::Black);
/// assert_eq!(occupied.player(), Some(Player::Black));
/// assert!(Cell::Empty.is_empty());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    /// An unoccupied intersection.
    Empty,
    /// An intersection holding the given player's stone.
    Stone(Player),
}

impl Cell {
    /// Whether this cell holds a stone of the given player.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Cell, Player};
    ///
    /// let cell = Cell::Stone(Player::Black);
    /// assert!(cell.is(Player::Black));
    /// assert!(!cell.is(Player::White));
    /// assert!(!Cell::Empty.is(Player::Black));
    /// ```
    #[inline]
    pub const fn is(self, player: Player) -> bool {
        matches!(self, Cell::Stone(p) if p as u8 == player as u8)
    }

    /// Whether this cell is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Cell, Player};
    ///
    /// assert!(Cell::Empty.is_empty());
    /// assert!(!Cell::Stone(Player::White).is_empty());
    /// ```
    #[inline]
    pub const fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }

    /// The player occupying this cell, if any.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Cell, Player};
    ///
    /// assert_eq!(Cell::Stone(Player::White).player(), Some(Player::White));
    /// assert_eq!(Cell::Empty.player(), None);
    /// ```
    #[inline]
    pub const fn player(self) -> Option<Player> {
        match self {
            Cell::Stone(p) => Some(p),
            Cell::Empty => None,
        }
    }

    /// The character used to render this cell: `.` for empty, otherwise the
    /// player's [`glyph`](Player::glyph).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Cell, Player};
    ///
    /// assert_eq!(Cell::Empty.glyph(), '.');
    /// assert_eq!(Cell::Stone(Player::Black).glyph(), 'X');
    /// assert_eq!(Cell::Stone(Player::White).glyph(), 'O');
    /// ```
    #[inline]
    pub const fn glyph(self) -> char {
        match self {
            Cell::Empty => '.',
            Cell::Stone(p) => p.glyph(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opponent_flips() {
        assert_eq!(Player::Black.opponent(), Player::White);
        assert_eq!(Player::White.opponent(), Player::Black);
    }

    #[test]
    fn cell_predicates() {
        let b = Cell::Stone(Player::Black);
        assert!(b.is(Player::Black));
        assert!(!b.is(Player::White));
        assert!(!b.is_empty());
        assert_eq!(b.player(), Some(Player::Black));
        assert!(Cell::Empty.is_empty());
        assert_eq!(Cell::Empty.player(), None);
    }
}
