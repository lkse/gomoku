//! Rule configuration. A [`RuleSet`] is a data-driven description of a variant
//! that the [`Game`](crate::Game) interprets; the heavier subsystems (Renju
//! forbidden moves, Pente captures, opening protocols) live in submodules and
//! are consulted only when the relevant flag is set.

pub mod capture;
pub mod opening;
pub mod renju;
pub mod win;

use crate::error::RuleSetError;
use crate::point::MAX_SIZE;

/// How a run longer than the win length is treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overline {
    /// A run of `win_length` *or more* wins (Freestyle, Caro).
    Win,
    /// Only a run of exactly `win_length` wins; longer runs do not (Standard,
    /// Renju — where an overline is additionally forbidden for Black).
    NoWin,
}

/// Pente-style capture configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capture {
    /// Number of captured *pairs* that wins the game.
    pub pairs_to_win: u8,
}

/// The opening protocol governing the first moves of the game.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opening {
    /// No restriction; plain alternating play from the first move.
    Free,
    /// Pro: black 1 at center, black 3 at least 3 lines from center.
    Pro,
    /// Long Pro: black 1 at center, black 3 at least 4 lines from center.
    LongPro,
    /// Swap: first player lays the opening three stones, second picks a color.
    Swap,
    /// Swap2: opening three, then swap / add-two-and-defer / take-black.
    Swap2,
    /// Renju forbidden-move rules with a free opening.
    FreeRenju,
    /// Yamaguchi: opening choice + announced 5th-move count, swap, then the
    /// proposer offers that many 5th moves for the opponent to choose from.
    Yamaguchi,
}

/// A complete description of a gomoku variant.
///
/// Construct one with a preset (e.g. [`RuleSet::standard`]) and adjust its
/// public fields as needed; the struct is `#[non_exhaustive]` so that new rule
/// options can be added without breaking callers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuleSet {
    /// Edge length of the (square) board, `5..=19`.
    pub board_size: u8,
    /// Stones in a row required to win (always 5 for standard variants).
    pub win_length: u8,
    /// Whether runs longer than `win_length` win.
    pub overline: Overline,
    /// Enforce Renju forbidden moves (double-three, double-four, overline) for Black.
    pub forbidden_black: bool,
    /// Caro: a winning run blocked by opponent stones at *both* ends does not win.
    pub caro_block_both_ends: bool,
    /// Pente capturing, if enabled.
    pub capture: Option<Capture>,
    /// Opening protocol.
    pub opening: Opening,
}

impl RuleSet {
    /// Freestyle gomoku: five *or more* in a row wins, no restrictions.
    pub fn freestyle() -> RuleSet {
        RuleSet {
            board_size: 15,
            win_length: 5,
            overline: Overline::Win,
            forbidden_black: false,
            caro_block_both_ends: false,
            capture: None,
            opening: Opening::Free,
        }
    }

    /// Standard gomoku: exactly five wins; an overline does not.
    pub fn standard() -> RuleSet {
        RuleSet {
            overline: Overline::NoWin,
            ..RuleSet::freestyle()
        }
    }

    /// Caro: five or more wins, but not if blocked by the opponent at both ends.
    /// Played on 15×15.
    pub fn caro() -> RuleSet {
        RuleSet {
            overline: Overline::Win,
            caro_block_both_ends: true,
            ..RuleSet::freestyle()
        }
    }

    /// Renju: Standard scoring plus Black's forbidden moves (double-three,
    /// double-four, overline). Opens freely (Free Renju); see
    /// [`renju_yamaguchi`](RuleSet::renju_yamaguchi) for the balanced opening.
    pub fn renju() -> RuleSet {
        RuleSet {
            overline: Overline::NoWin,
            forbidden_black: true,
            opening: Opening::FreeRenju,
            ..RuleSet::freestyle()
        }
    }

    /// Omok (Korean): exactly five wins (no overline) on a 19×19 board.
    pub fn omok() -> RuleSet {
        RuleSet {
            board_size: 19,
            ..RuleSet::standard()
        }
    }

    /// Renju with the Yamaguchi balanced opening.
    pub fn renju_yamaguchi() -> RuleSet {
        RuleSet {
            opening: Opening::Yamaguchi,
            ..RuleSet::renju()
        }
    }

    /// Standard gomoku with the Swap opening (place three, then choose a color).
    pub fn swap() -> RuleSet {
        RuleSet {
            opening: Opening::Swap,
            ..RuleSet::standard()
        }
    }

    /// Standard gomoku with the Swap2 opening.
    pub fn swap2() -> RuleSet {
        RuleSet {
            opening: Opening::Swap2,
            ..RuleSet::standard()
        }
    }

    /// Standard gomoku with the Pro opening (Black 1 center, Black 3 ≥3 away).
    pub fn pro() -> RuleSet {
        RuleSet {
            opening: Opening::Pro,
            ..RuleSet::standard()
        }
    }

    /// Standard gomoku with the Long Pro opening (Black 3 ≥4 away).
    pub fn long_pro() -> RuleSet {
        RuleSet {
            opening: Opening::LongPro,
            ..RuleSet::standard()
        }
    }

    /// Pente: five in a row *or* five captured pairs wins, on a 19×19 board.
    pub fn pente() -> RuleSet {
        RuleSet {
            board_size: 19,
            capture: Some(Capture { pairs_to_win: 5 }),
            ..RuleSet::freestyle()
        }
    }

    /// Check that the configuration is internally consistent.
    ///
    /// # Errors
    ///
    /// Returns a [`RuleSetError`] if the board size is out of range, the win
    /// length is not within `2..=board_size`, or capturing is enabled with a
    /// zero pairs-to-win target.
    pub fn validate(&self) -> Result<(), RuleSetError> {
        if !(5..=MAX_SIZE).contains(&self.board_size) {
            return Err(RuleSetError::BoardSize(self.board_size));
        }
        if self.win_length < 2 || self.win_length > self.board_size {
            return Err(RuleSetError::WinLength(self.win_length));
        }
        if let Some(c) = self.capture {
            if c.pairs_to_win == 0 {
                return Err(RuleSetError::ZeroCapturePairs);
            }
        }
        Ok(())
    }

    /// Return a copy of these rules with a different board size.
    ///
    /// # Panics
    ///
    /// Panics unless `5 <= size <= 19`.
    #[must_use]
    pub fn with_board_size(mut self, size: u8) -> RuleSet {
        assert!(
            (5..=MAX_SIZE).contains(&size),
            "board size must be in 5..=19, got {size}"
        );
        self.board_size = size;
        self
    }
}
