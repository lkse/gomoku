//! Dependency-free serialization: a FEN-like board snapshot and a replayable
//! move list. No `serde`, no allocations beyond the produced `String`/`Game`.

use crate::board::Board;
use crate::error::MoveError;
use crate::game::Game;
use crate::point::{Point, MAX_SIZE};
use crate::rules::RuleSet;
use crate::stone::Player;
use std::fmt;

/// Failure parsing a board FEN string.
///
/// Returned by [`Board::from_fen`].
///
/// # Examples
///
/// ```
/// use gomoku::{Board, FenError};
///
/// assert_eq!(Board::from_fen("nope"), Err(FenError::BadSize));
/// assert_eq!(Board::from_fen("4:...."), Err(FenError::BadSize)); // size below 5
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenError {
    /// Missing the `size:` prefix, or the size is out of range.
    BadSize,
    /// A cell character other than `.`, `X`, or `O`.
    BadChar(char),
    /// The row count or a row's length does not match the declared size.
    BadShape,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::BadSize => f.write_str("missing or invalid board size"),
            FenError::BadChar(c) => write!(f, "invalid cell character '{c}'"),
            FenError::BadShape => f.write_str("row count or length does not match the board size"),
        }
    }
}

impl std::error::Error for FenError {}

/// Failure replaying a move list.
///
/// Returned by [`Game::from_move_list`].
///
/// # Examples
///
/// ```
/// use gomoku::{Game, MoveListError, RuleSet};
///
/// // `??` is not a coordinate, so replay fails on that token.
/// let err = Game::from_move_list(RuleSet::standard(), "h8 ??").unwrap_err();
/// assert_eq!(err, MoveListError::BadCoordinate("??".to_string()));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveListError {
    /// A token that is not a valid coordinate (e.g. `"h8"`).
    BadCoordinate(String),
    /// A coordinate that was rejected during replay.
    Illegal(MoveError),
}

impl fmt::Display for MoveListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveListError::BadCoordinate(s) => write!(f, "invalid coordinate '{s}'"),
            MoveListError::Illegal(e) => write!(f, "illegal move during replay: {e}"),
        }
    }
}

impl std::error::Error for MoveListError {}

impl Board {
    /// Encode the board as `"<size>:<row>/<row>/…"`, top row first. Cells are
    /// `.` (empty), `X` (Black), `O` (White).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Player, Point};
    ///
    /// let mut board = Board::new(5);
    /// board.place(Player::Black, Point::new(0, 0)); // bottom-left corner
    ///
    /// // Five rows, top-first; the stone sits in the last (bottom) row.
    /// assert_eq!(board.to_fen(), "5:...../...../...../...../X....");
    /// ```
    #[must_use]
    pub fn to_fen(&self) -> String {
        let size = self.size();
        let mut s = format!("{size}:");
        for y in (0..size).rev() {
            if y != size - 1 {
                s.push('/');
            }
            for x in 0..size {
                s.push(self.get(Point::new(x, y)).glyph());
            }
        }
        s
    }

    /// Parse a board produced by [`Board::to_fen`].
    ///
    /// # Errors
    ///
    /// Returns [`FenError`] if the size prefix is missing or out of range, the
    /// row count or a row length does not match the size, or a cell character
    /// is not one of `.`, `X`, `O`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Board, Player, Point};
    ///
    /// let mut original = Board::new(15);
    /// original.place(Player::Black, Point::new(7, 7));
    ///
    /// // `to_fen` and `from_fen` are inverses.
    /// let restored = Board::from_fen(&original.to_fen())?;
    /// assert_eq!(restored, original);
    /// # Ok::<(), gomoku::FenError>(())
    /// ```
    pub fn from_fen(s: &str) -> Result<Board, FenError> {
        let (size_str, rows_str) = s.split_once(':').ok_or(FenError::BadSize)?;
        let size: u8 = size_str.trim().parse().map_err(|_| FenError::BadSize)?;
        if !(5..=MAX_SIZE).contains(&size) {
            return Err(FenError::BadSize);
        }

        let rows: Vec<&str> = rows_str.split('/').collect();
        if rows.len() != size as usize {
            return Err(FenError::BadShape);
        }

        let mut board = Board::new(size);
        for (ri, row) in rows.iter().enumerate() {
            if row.chars().count() != size as usize {
                return Err(FenError::BadShape);
            }
            // The first row is the top of the board (highest y).
            let y = size - 1 - ri as u8;
            for (x, ch) in row.chars().enumerate() {
                let p = Point::new(x as u8, y);
                match ch {
                    '.' => {}
                    'X' => board.place(Player::Black, p),
                    'O' => board.place(Player::White, p),
                    other => return Err(FenError::BadChar(other)),
                }
            }
        }
        Ok(board)
    }
}

impl Game {
    /// The move history as a space-separated list of algebraic coordinates,
    /// e.g. `"h8 i9 g7"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Game, Point, RuleSet};
    ///
    /// let mut game = Game::new(RuleSet::standard());
    /// game.play(Point::new(7, 7))?; // h8
    /// game.play(Point::new(7, 8))?; // h9
    /// assert_eq!(game.to_move_list(), "h8 h9");
    /// # Ok::<(), gomoku::MoveError>(())
    /// ```
    #[must_use]
    pub fn to_move_list(&self) -> String {
        self.moves()
            .iter()
            .map(Point::to_string)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Rebuild a game by replaying a move list under `rules`.
    ///
    /// This drives [`Game::play`] for each coordinate, so it works for openings
    /// made entirely of stone placements (Free, Free Renju, Pro, Long Pro). It
    /// cannot replay protocols with interactive decisions (Swap/Swap2/Yamaguchi),
    /// which need their decision methods rather than a plain move list.
    ///
    /// # Errors
    ///
    /// Returns [`MoveListError::BadCoordinate`] for a token that is not valid
    /// algebraic notation, or [`MoveListError::Illegal`] if a move is rejected
    /// during replay.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Game, RuleSet};
    ///
    /// let game = Game::from_move_list(RuleSet::standard(), "h8 h1 i8 i1 j8")?;
    /// assert_eq!(game.move_count(), 5);
    /// // The reconstructed game replays back to the same notation.
    /// assert_eq!(game.to_move_list(), "h8 h1 i8 i1 j8");
    /// # Ok::<(), gomoku::MoveListError>(())
    /// ```
    pub fn from_move_list(rules: RuleSet, list: &str) -> Result<Game, MoveListError> {
        let mut game = Game::new(rules);
        for tok in list.split_whitespace() {
            let p =
                Point::parse(tok).ok_or_else(|| MoveListError::BadCoordinate(tok.to_string()))?;
            game.play(p).map_err(MoveListError::Illegal)?;
        }
        Ok(game)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_fen_round_trips() {
        let mut b = Board::new(15);
        b.place(Player::Black, Point::new(7, 7));
        b.place(Player::White, Point::new(7, 8));
        b.place(Player::Black, Point::new(0, 0));
        let fen = b.to_fen();
        let back = Board::from_fen(&fen).unwrap();
        assert_eq!(b, back);
        assert_eq!(back.to_fen(), fen);
    }

    #[test]
    fn fen_rejects_bad_input() {
        assert_eq!(Board::from_fen("nope"), Err(FenError::BadSize));
        assert_eq!(Board::from_fen("4:...."), Err(FenError::BadSize));
        assert!(matches!(Board::from_fen("15:XQ"), Err(FenError::BadShape)));
    }

    #[test]
    fn move_list_replays_to_same_board() {
        let mut g = Game::new(RuleSet::standard());
        for &(x, y) in &[(7, 7), (7, 8), (8, 8), (8, 9), (9, 9)] {
            g.play(Point::new(x, y)).unwrap();
        }
        let list = g.to_move_list();
        let replayed = Game::from_move_list(RuleSet::standard(), &list).unwrap();
        assert_eq!(replayed.board().to_fen(), g.board().to_fen());
        assert_eq!(replayed.to_move_list(), list);
    }

    #[test]
    fn move_list_reports_bad_token() {
        match Game::from_move_list(RuleSet::standard(), "h8 ??") {
            Err(e) => assert_eq!(e, MoveListError::BadCoordinate("??".to_string())),
            Ok(_) => panic!("expected a parse error"),
        }
    }
}
