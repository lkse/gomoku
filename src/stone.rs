//! The two players and the contents of a board cell.

/// One of the two players. Black always moves first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    /// The first player. Renders as `X` and always moves first.
    Black,
    /// The second player. Renders as `O`.
    White,
}

impl Player {
    /// The other player.
    #[inline]
    pub const fn opponent(self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    /// The single character used to render this player's stones (`X` / `O`).
    #[inline]
    pub const fn glyph(self) -> char {
        match self {
            Player::Black => 'X',
            Player::White => 'O',
        }
    }
}

/// The contents of a single intersection on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    /// An unoccupied intersection.
    Empty,
    /// An intersection holding the given player's stone.
    Stone(Player),
}

impl Cell {
    /// Whether this cell holds a stone of the given player.
    #[inline]
    pub const fn is(self, player: Player) -> bool {
        matches!(self, Cell::Stone(p) if p as u8 == player as u8)
    }

    /// Whether this cell is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }

    /// The player occupying this cell, if any.
    #[inline]
    pub const fn player(self) -> Option<Player> {
        match self {
            Cell::Stone(p) => Some(p),
            Cell::Empty => None,
        }
    }

    /// The character used to render this cell.
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
