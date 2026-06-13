//! A fast, zero-dependency game and rule engine for gomoku narabe (五目並べ)
//! and its many variants.
//!
//! `gomoku` maintains board state and arbitrates the rules. It is an engine,
//! not a player: there is no AI or move search. The intended use is as the
//! authoritative rule layer beneath a UI, a server, or a search-based bot.
//!
//! # Supported variants
//!
//! Select a variant with a [`RuleSet`] preset, or assemble one field by field:
//!
//! | Preset | Win condition | Notes |
//! |--------|---------------|-------|
//! | [`RuleSet::freestyle`] | five or more in a row | no restrictions |
//! | [`RuleSet::standard`]  | exactly five | overline does not win |
//! | [`RuleSet::renju`]     | exactly five | Black forbidden moves; free opening |
//! | [`RuleSet::caro`]      | five or more | must not be blocked at both ends |
//! | [`RuleSet::omok`]      | exactly five | 19×19 |
//! | [`RuleSet::pente`]     | five in a row, or five captured pairs | custodial captures |
//!
//! Opening protocols — Free, Pro, Long Pro, Swap, Swap2, and Yamaguchi — are
//! available via [`Opening`], backed by the catalog of all 26 canonical Renju
//! openings in [`OpeningName`].
//!
//! # Design and performance
//!
//! Occupancy is stored as two per-color bitboards, so membership tests and
//! whole-board scans are a handful of word operations. Because only the most
//! recently placed stone can complete a line, win and threat detection examine
//! just the four axes through that stone — each move is evaluated in O(1)
//! regardless of board size. On commodity hardware a single move is checked in
//! roughly 25 ns (Renju, with its recursive forbidden-move analysis, is the
//! exception). See `cargo bench` and the `bench` example for measurements.
//! # Examples
//!
//! ```
//! use gomoku::{Game, Player, Point, RuleSet, Status};
//!
//! let mut game = Game::new(RuleSet::standard());
//! // Black builds a five along a row; White answers elsewhere.
//! for x in 0..4 {
//!     game.play(Point::new(x, 7))?; // Black
//!     game.play(Point::new(x, 0))?; // White
//! }
//! let outcome = game.play(Point::new(4, 7))?;
//! assert_eq!(outcome.status, Status::Win(Player::Black));
//! assert_eq!(game.winner(), Some(Player::Black));
//! # Ok::<(), gomoku::MoveError>(())
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod board;
mod error;
mod game;
mod point;
mod rules;
mod serialize;
mod stone;

pub use board::Board;
pub use error::{ForbiddenKind, MoveError, OpeningError, RuleSetError};
pub use game::{Game, MoveOutcome, Status};
pub use point::{Point, MAX_SIZE};
pub use rules::opening::{Constraint, OpeningAction, OpeningName, Swap2Choice, ALL_OPENINGS};
pub use rules::{Capture, Opening, Overline, RuleSet};
pub use serialize::{FenError, MoveListError};
pub use stone::{Cell, Player};
