//! Errors returned when a move is rejected.

use crate::point::Point;
use std::fmt;

/// Why a Renju move is forbidden for Black.
///
/// Carried inside [`MoveError::Forbidden`] when a Black move is rejected under
/// Renju rules.
///
/// # Examples
///
/// ```
/// use gomoku::ForbiddenKind;
///
/// assert_eq!(ForbiddenKind::DoubleThree.to_string(), "double-three (3-3)");
/// assert_eq!(ForbiddenKind::Overline.to_string(), "overline (long row)");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForbiddenKind {
    /// Two open threes created at once (3-3).
    DoubleThree,
    /// Two fours created at once (4-4).
    DoubleFour,
    /// A line of six or more (長連).
    Overline,
}

impl fmt::Display for ForbiddenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ForbiddenKind::DoubleThree => "double-three (3-3)",
            ForbiddenKind::DoubleFour => "double-four (4-4)",
            ForbiddenKind::Overline => "overline (long row)",
        };
        f.write_str(s)
    }
}

/// Why a [`RuleSet`](crate::RuleSet) is not a valid configuration.
///
/// Returned by [`RuleSet::validate`](crate::RuleSet::validate) and
/// [`Game::try_new`](crate::Game::try_new).
///
/// # Examples
///
/// ```
/// use gomoku::{RuleSet, RuleSetError};
///
/// let mut rules = RuleSet::standard();
/// rules.board_size = 99; // out of the supported 5..=19 range
/// assert_eq!(rules.validate(), Err(RuleSetError::BoardSize(99)));
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleSetError {
    /// `board_size` is outside the supported range `5..=19`.
    BoardSize(u8),
    /// `win_length` is less than 2 or greater than `board_size`.
    WinLength(u8),
    /// Capturing is enabled but `pairs_to_win` is zero.
    ZeroCapturePairs,
}

impl fmt::Display for RuleSetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleSetError::BoardSize(n) => write!(f, "board size {n} is not in 5..=19"),
            RuleSetError::WinLength(n) => write!(f, "win length {n} must be in 2..=board_size"),
            RuleSetError::ZeroCapturePairs => {
                f.write_str("capture pairs-to-win must be at least 1")
            }
        }
    }
}

impl std::error::Error for RuleSetError {}

/// Why an opening-protocol action was rejected.
///
/// Carried inside [`MoveError::Opening`] when a placement or decision method is
/// called at the wrong point in an opening protocol.
///
/// # Examples
///
/// ```
/// use gomoku::{Game, MoveError, OpeningError, Player, RuleSet};
///
/// // Standard rules have no opening decisions, so a color choice is unexpected.
/// let mut game = Game::new(RuleSet::standard());
/// assert_eq!(
///     game.choose_color(Player::Black),
///     Err(MoveError::Opening(OpeningError::UnexpectedColorChoice)),
/// );
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpeningError {
    /// A stone placement was attempted while a non-placement decision is pending.
    DecisionRequired,
    /// A placement broke the protocol's location restriction (center / distance).
    PlacementRestricted,
    /// A color choice was made when none was expected.
    UnexpectedColorChoice,
    /// A Swap2 decision was made when none was expected.
    UnexpectedSwap2Decision,
    /// A 5th-move count announcement was made when none was expected.
    UnexpectedAnnouncement,
    /// The announced 5th-move count was zero.
    ZeroCount,
    /// A 5th-move proposal was made when none was expected.
    UnexpectedProposal,
    /// The number of proposed moves did not match the announced count.
    WrongProposalCount,
    /// The proposed moves contained a duplicate.
    DuplicateProposal,
    /// A 5th-move selection was made when none was expected.
    UnexpectedSelection,
    /// The selected move was not among those proposed.
    NotProposed,
}

impl fmt::Display for OpeningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            OpeningError::DecisionRequired => "an opening decision is required before playing",
            OpeningError::PlacementRestricted => "move violates the opening placement restriction",
            OpeningError::UnexpectedColorChoice => "no color choice is expected now",
            OpeningError::UnexpectedSwap2Decision => "no Swap2 decision is expected now",
            OpeningError::UnexpectedAnnouncement => "no count announcement is expected now",
            OpeningError::ZeroCount => "the announced count must be at least 1",
            OpeningError::UnexpectedProposal => "no 5th-move proposal is expected now",
            OpeningError::WrongProposalCount => "wrong number of proposed moves",
            OpeningError::DuplicateProposal => "proposed moves must be distinct",
            OpeningError::UnexpectedSelection => "no 5th-move selection is expected now",
            OpeningError::NotProposed => "the chosen move was not among the proposals",
        };
        f.write_str(s)
    }
}

impl std::error::Error for OpeningError {}

/// The reason a call to [`Game::play`](crate::Game::play) (or an opening action)
/// was rejected. No board state is changed when an error is returned.
///
/// # Examples
///
/// ```
/// use gomoku::{Game, MoveError, Point, RuleSet};
///
/// let mut game = Game::new(RuleSet::standard());
/// game.play(Point::new(7, 7))?;
///
/// // Replaying an occupied point is rejected and leaves the game untouched.
/// assert_eq!(
///     game.play(Point::new(7, 7)),
///     Err(MoveError::Occupied(Point::new(7, 7))),
/// );
/// assert_eq!(game.move_count(), 1);
/// # Ok::<(), MoveError>(())
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveError {
    /// The point lies outside the board.
    OutOfBounds(Point),
    /// The point is already occupied.
    Occupied(Point),
    /// The game has already ended.
    GameOver,
    /// A Renju forbidden move for Black.
    Forbidden(ForbiddenKind),
    /// The move or decision conflicts with the active opening protocol.
    Opening(OpeningError),
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::OutOfBounds(p) => write!(f, "point {p} is off the board"),
            MoveError::Occupied(p) => write!(f, "point {p} is already occupied"),
            MoveError::GameOver => f.write_str("the game is already over"),
            MoveError::Forbidden(k) => write!(f, "forbidden move for Black: {k}"),
            MoveError::Opening(e) => write!(f, "opening rule violated: {e}"),
        }
    }
}

impl std::error::Error for MoveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MoveError::Opening(e) => Some(e),
            _ => None,
        }
    }
}
