//! The game: board state, turn order, move legality, and history.

use crate::board::Board;
use crate::error::{MoveError, OpeningError, RuleSetError};
use crate::point::Point;
use crate::rules::opening::{Constraint, OpeningAction, Swap2Choice};
use crate::rules::{win, Opening, RuleSet};
use crate::stone::Player;

/// The current outcome of a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Play continues.
    InProgress,
    /// `player` has won.
    Win(Player),
    /// The board filled with no winner.
    Draw,
}

impl Status {
    /// Whether the game has finished.
    #[inline]
    pub fn is_over(self) -> bool {
        !matches!(self, Status::InProgress)
    }
}

/// A record of one played move, retained so it can be undone.
#[derive(Debug, Clone)]
struct PlayedMove {
    point: Point,
    player: Player,
    /// Opponent stones removed by this move (Pente captures).
    captured: Vec<Point>,
    /// Opening step before this move, restored on undo.
    opening_step_before: u8,
}

/// Sentinel `opening_step` meaning the opening is complete.
const OPENING_DONE: u8 = u8::MAX;

/// The result of a successful [`Game::play`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveOutcome {
    /// Game status after the move.
    pub status: Status,
    /// Opponent stones captured by the move (empty unless capturing is enabled).
    pub captured: Vec<Point>,
}

/// A single game of gomoku under a fixed [`RuleSet`].
#[derive(Clone, Debug)]
pub struct Game {
    rules: RuleSet,
    board: Board,
    to_move: Player,
    status: Status,
    history: Vec<PlayedMove>,
    /// Captured-pair counts, indexed by `player as usize` (Black = 0, White = 1).
    captures: [u16; 2],
    /// Cursor into the active opening protocol's step sequence.
    opening_step: u8,
    /// Yamaguchi: announced number of 5th-move candidates.
    opening_count: u8,
    /// Yamaguchi: the currently offered 5th-move candidates.
    opening_proposals: Vec<Point>,
    /// The color chosen by a swap-style decision, once made.
    opening_color_choice: Option<Player>,
}

impl Game {
    /// Start a new game under `rules`, validating the configuration first.
    ///
    /// Prefer this over [`Game::new`] when the rules come from untrusted input.
    ///
    /// # Errors
    ///
    /// Returns a [`RuleSetError`] if `rules` is not a valid configuration; see
    /// [`RuleSet::validate`].
    pub fn try_new(rules: RuleSet) -> Result<Game, RuleSetError> {
        rules.validate()?;
        Ok(Game::new(rules))
    }

    /// Start a new game under `rules`. Black moves first.
    ///
    /// # Panics
    ///
    /// Panics if `rules` is not valid (see [`RuleSet::validate`]); most importantly
    /// if `rules.board_size` is outside `5..=19`. Use [`Game::try_new`] to handle
    /// invalid configurations without panicking.
    #[must_use]
    pub fn new(rules: RuleSet) -> Game {
        Game {
            board: Board::new(rules.board_size),
            rules,
            to_move: Player::Black,
            status: Status::InProgress,
            history: Vec::new(),
            captures: [0, 0],
            opening_step: 0,
            opening_count: 0,
            opening_proposals: Vec::new(),
            opening_color_choice: None,
        }
    }

    /// The rules in force.
    #[inline]
    pub fn rules(&self) -> &RuleSet {
        &self.rules
    }

    /// The board.
    #[inline]
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Whose turn it is.
    #[inline]
    pub fn to_move(&self) -> Player {
        self.to_move
    }

    /// Current status.
    #[inline]
    pub fn status(&self) -> Status {
        self.status
    }

    /// Number of pairs `player` has captured (always 0 unless capturing is on).
    #[inline]
    pub fn captures(&self, player: Player) -> u16 {
        self.captures[player as usize]
    }

    /// The winner, if the game has been won.
    #[inline]
    #[must_use]
    pub fn winner(&self) -> Option<Player> {
        match self.status {
            Status::Win(p) => Some(p),
            _ => None,
        }
    }

    /// Number of moves played so far.
    #[inline]
    pub fn move_count(&self) -> usize {
        self.history.len()
    }

    /// The points played so far, in order.
    #[must_use]
    pub fn moves(&self) -> Vec<Point> {
        self.history.iter().map(|m| m.point).collect()
    }

    /// What the active opening protocol expects next. Returns
    /// [`OpeningAction::None`] once the opening is over (or for free openings).
    #[must_use]
    pub fn opening_action(&self) -> OpeningAction {
        use Constraint::Anywhere;
        use Player::{Black, White};
        let place = |c, k| OpeningAction::PlaceStone {
            color: c,
            constraint: k,
        };
        match self.rules.opening {
            Opening::Free | Opening::FreeRenju => OpeningAction::None,
            Opening::Pro => self.pro_action(3),
            Opening::LongPro => self.pro_action(4),
            Opening::Swap => match self.opening_step {
                0 | 2 => place(Black, Anywhere),
                1 => place(White, Anywhere),
                3 => OpeningAction::ChooseColor,
                _ => OpeningAction::None,
            },
            Opening::Swap2 => match self.opening_step {
                0 | 2 | 5 => place(Black, Anywhere),
                1 | 4 => place(White, Anywhere),
                3 => OpeningAction::Swap2Decision,
                6 => OpeningAction::ChooseColor,
                _ => OpeningAction::None,
            },
            Opening::Yamaguchi => match self.opening_step {
                0 | 2 => place(Black, Anywhere),
                1 | 5 => place(White, Anywhere),
                3 => OpeningAction::AnnounceCount,
                4 => OpeningAction::ChooseColor,
                6 => OpeningAction::ProposeFifths {
                    count: self.opening_count,
                },
                7 => OpeningAction::SelectFifth {
                    options: self.opening_proposals.clone(),
                },
                _ => OpeningAction::None,
            },
        }
    }

    /// Pro / Long Pro share a sequence differing only in the stone-3 distance.
    fn pro_action(&self, distance: u8) -> OpeningAction {
        use Player::{Black, White};
        match self.opening_step {
            0 => OpeningAction::PlaceStone {
                color: Black,
                constraint: Constraint::Center,
            },
            1 => OpeningAction::PlaceStone {
                color: White,
                constraint: Constraint::Anywhere,
            },
            2 => OpeningAction::PlaceStone {
                color: Black,
                constraint: Constraint::MinDistance(distance),
            },
            _ => OpeningAction::None,
        }
    }

    /// The color chosen by a swap-style decision, if one has been made.
    #[inline]
    pub fn opening_color_choice(&self) -> Option<Player> {
        self.opening_color_choice
    }

    /// Bounds / occupancy / Renju-forbidden check, ignoring opening sequencing.
    fn move_playable(&self, p: Point) -> Result<(), MoveError> {
        if self.status.is_over() {
            return Err(MoveError::GameOver);
        }
        if !self.board.in_bounds(p) {
            return Err(MoveError::OutOfBounds(p));
        }
        if !self.board.is_empty(p) {
            return Err(MoveError::Occupied(p));
        }
        if self.rules.forbidden_black && self.to_move == Player::Black {
            if let Some(kind) = crate::rules::renju::forbidden(&self.board, p) {
                return Err(MoveError::Forbidden(kind));
            }
        }
        Ok(())
    }

    /// Validate a move for the current player without mutating the game,
    /// including any opening-protocol gating.
    fn check_legal(&self, p: Point) -> Result<(), MoveError> {
        match self.opening_action() {
            OpeningAction::None => {}
            OpeningAction::PlaceStone { constraint, .. } => {
                if !constraint.allows(p, self.board.center()) {
                    return Err(MoveError::Opening(OpeningError::PlacementRestricted));
                }
            }
            _ => return Err(MoveError::Opening(OpeningError::DecisionRequired)),
        }
        self.move_playable(p)
    }

    /// Whether the current player may legally play at `p`.
    #[inline]
    #[must_use]
    pub fn is_legal(&self, p: Point) -> bool {
        self.check_legal(p).is_ok()
    }

    /// All points the current player may legally play.
    ///
    /// Returns an empty vector when the game is over or an opening decision is
    /// pending. For Renju this excludes forbidden points for Black.
    #[must_use]
    pub fn legal_moves(&self) -> Vec<Point> {
        if self.status.is_over() {
            return Vec::new();
        }
        self.board.points().filter(|&p| self.is_legal(p)).collect()
    }

    /// Play a stone for the current player at `p`.
    ///
    /// On success the stone is placed, captures (if any) are resolved, the game
    /// status is updated, and the turn passes to the opponent.
    ///
    /// # Errors
    ///
    /// Returns [`MoveError`] without modifying the game if the game is over, the
    /// point is off-board or occupied, the move is forbidden under Renju, or an
    /// opening-protocol restriction or pending decision disallows it.
    pub fn play(&mut self, p: Point) -> Result<MoveOutcome, MoveError> {
        self.check_legal(p)?;

        let was_opening_place = matches!(self.opening_action(), OpeningAction::PlaceStone { .. });
        let opening_step_before = self.opening_step;
        let player = self.to_move;
        self.board.place(player, p);

        let captured = if self.rules.capture.is_some() {
            crate::rules::capture::resolve(&mut self.board, p, player)
        } else {
            Vec::new()
        };
        self.captures[player as usize] += (captured.len() / 2) as u16;

        let capture_win = self
            .rules
            .capture
            .is_some_and(|c| self.captures[player as usize] >= c.pairs_to_win as u16);

        self.status = if win::is_win(&self.board, p, player, &self.rules) || capture_win {
            Status::Win(player)
        } else if self.board.is_full() {
            Status::Draw
        } else {
            Status::InProgress
        };

        self.history.push(PlayedMove {
            point: p,
            player,
            captured: captured.clone(),
            opening_step_before,
        });
        self.to_move = player.opponent();
        if was_opening_place {
            self.opening_step = self.opening_step.saturating_add(1);
        }

        Ok(MoveOutcome {
            status: self.status,
            captured,
        })
    }

    /// Undo the most recent move, restoring any captured stones and the turn.
    /// Returns the point that was undone, or `None` if no moves had been made.
    pub fn undo(&mut self) -> Option<Point> {
        let last = self.history.pop()?;
        self.board.clear(last.point);
        for &c in &last.captured {
            self.board.place(last.player.opponent(), c);
        }
        self.captures[last.player as usize] -= (last.captured.len() / 2) as u16;
        self.opening_step = last.opening_step_before;
        self.to_move = last.player;
        self.status = Status::InProgress;
        Some(last.point)
    }

    /// Choose which color to continue as, for a swap-style opening decision.
    ///
    /// # Errors
    ///
    /// Returns [`OpeningError::UnexpectedColorChoice`] if no color choice is due.
    pub fn choose_color(&mut self, color: Player) -> Result<(), MoveError> {
        if self.opening_action() != OpeningAction::ChooseColor {
            return Err(MoveError::Opening(OpeningError::UnexpectedColorChoice));
        }
        self.opening_color_choice = Some(color);
        self.opening_step = self.opening_step.saturating_add(1);
        Ok(())
    }

    /// Make the Swap2 three-way decision.
    ///
    /// # Errors
    ///
    /// Returns [`OpeningError::UnexpectedSwap2Decision`] if no Swap2 decision is due.
    pub fn swap2_decision(&mut self, choice: Swap2Choice) -> Result<(), MoveError> {
        if self.opening_action() != OpeningAction::Swap2Decision {
            return Err(MoveError::Opening(OpeningError::UnexpectedSwap2Decision));
        }
        match choice {
            Swap2Choice::PlayWhite => {
                self.opening_color_choice = Some(Player::White);
                self.opening_step = OPENING_DONE;
            }
            Swap2Choice::SwapToBlack => {
                self.opening_color_choice = Some(Player::Black);
                self.opening_step = OPENING_DONE;
            }
            Swap2Choice::PlaceTwoMore => self.opening_step = 4,
        }
        Ok(())
    }

    /// Yamaguchi: announce how many candidate 5th moves Black will offer.
    ///
    /// # Errors
    ///
    /// Returns [`OpeningError::UnexpectedAnnouncement`] if no announcement is due,
    /// or [`OpeningError::ZeroCount`] if `count` is zero.
    pub fn announce_fifth_count(&mut self, count: u8) -> Result<(), MoveError> {
        if self.opening_action() != OpeningAction::AnnounceCount {
            return Err(MoveError::Opening(OpeningError::UnexpectedAnnouncement));
        }
        if count == 0 {
            return Err(MoveError::Opening(OpeningError::ZeroCount));
        }
        self.opening_count = count;
        self.opening_step = self.opening_step.saturating_add(1);
        Ok(())
    }

    /// Yamaguchi: offer the announced number of candidate 5th moves. Each must
    /// be a distinct, currently legal move for Black.
    ///
    /// # Errors
    ///
    /// Returns an [`OpeningError`] if no proposal is due, the count is wrong, or a
    /// duplicate is offered; or the [`MoveError`] from the first illegal candidate.
    pub fn propose_fifths(&mut self, points: &[Point]) -> Result<(), MoveError> {
        let OpeningAction::ProposeFifths { count } = self.opening_action() else {
            return Err(MoveError::Opening(OpeningError::UnexpectedProposal));
        };
        if points.len() != count as usize {
            return Err(MoveError::Opening(OpeningError::WrongProposalCount));
        }
        for (i, &p) in points.iter().enumerate() {
            if points[..i].contains(&p) {
                return Err(MoveError::Opening(OpeningError::DuplicateProposal));
            }
            self.move_playable(p)?;
        }
        self.opening_proposals = points.to_vec();
        self.opening_step = self.opening_step.saturating_add(1);
        Ok(())
    }

    /// Yamaguchi: select one of the offered 5th moves and play it (as Black),
    /// ending the opening.
    ///
    /// # Errors
    ///
    /// Returns [`OpeningError::UnexpectedSelection`] if no selection is due, or
    /// [`OpeningError::NotProposed`] if `p` was not among the proposals.
    pub fn choose_fifth(&mut self, p: Point) -> Result<MoveOutcome, MoveError> {
        let OpeningAction::SelectFifth { options } = self.opening_action() else {
            return Err(MoveError::Opening(OpeningError::UnexpectedSelection));
        };
        if !options.contains(&p) {
            return Err(MoveError::Opening(OpeningError::NotProposed));
        }
        self.opening_step = OPENING_DONE;
        self.opening_proposals.clear();
        self.play(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternates_and_detects_win() {
        let mut g = Game::new(RuleSet::standard());
        // Black builds a horizontal five while White plays elsewhere.
        let blacks = [(2, 2), (3, 2), (4, 2), (5, 2), (6, 2)];
        let whites = [(2, 10), (3, 10), (4, 10), (5, 10)];
        for i in 0..4 {
            assert_eq!(g.to_move(), Player::Black);
            let out = g.play(Point::new(blacks[i].0, blacks[i].1)).unwrap();
            assert_eq!(out.status, Status::InProgress);
            assert_eq!(g.to_move(), Player::White);
            g.play(Point::new(whites[i].0, whites[i].1)).unwrap();
        }
        let out = g.play(Point::new(6, 2)).unwrap();
        assert_eq!(out.status, Status::Win(Player::Black));
        assert_eq!(g.winner(), Some(Player::Black));
        assert!(g.status().is_over());
    }

    #[test]
    fn rejects_illegal_moves() {
        let mut g = Game::new(RuleSet::standard());
        g.play(Point::new(7, 7)).unwrap();
        assert_eq!(
            g.play(Point::new(7, 7)),
            Err(MoveError::Occupied(Point::new(7, 7)))
        );
        assert_eq!(
            g.play(Point::new(20, 0)),
            Err(MoveError::OutOfBounds(Point::new(20, 0)))
        );
        assert!(!g.is_legal(Point::new(7, 7)));
    }

    #[test]
    fn undo_restores_turn_and_status() {
        let mut g = Game::new(RuleSet::freestyle());
        for x in 2..6 {
            g.play(Point::new(x, 2)).unwrap(); // black
            g.play(Point::new(x, 9)).unwrap(); // white
        }
        let out = g.play(Point::new(6, 2)).unwrap(); // black wins
        assert_eq!(out.status, Status::Win(Player::Black));
        assert_eq!(g.undo(), Some(Point::new(6, 2)));
        assert_eq!(g.status(), Status::InProgress);
        assert_eq!(g.to_move(), Player::Black);
        assert!(g.board().is_empty(Point::new(6, 2)));
    }

    #[test]
    fn no_moves_after_game_over() {
        let mut g = Game::new(RuleSet::freestyle());
        for x in 2..6 {
            g.play(Point::new(x, 2)).unwrap();
            g.play(Point::new(x, 9)).unwrap();
        }
        g.play(Point::new(6, 2)).unwrap(); // black wins
        assert_eq!(g.play(Point::new(0, 0)), Err(MoveError::GameOver));
        assert!(g.legal_moves().is_empty());
    }
}
