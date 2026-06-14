//! Opening protocols and the catalog of the 26 canonical Renju openings.
//!
//! The named openings are constructed from the standard direct/indirect scheme:
//! stone 1 sits on the center, stone 2 is placed *orthogonally* adjacent for a
//! **direct** (直) opening or *diagonally* adjacent for an **indirect** (間)
//! opening, and stone 3 takes one of the symmetry-distinct cells within the
//! central 5×5 area. Each scheme yields exactly 13 distinct stone-3 placements,
//! for 26 openings in total.
//!
//! The name → position assignment follows this construction (it is stable and
//! round-trips through [`OpeningName::identify`]); it is not guaranteed to match
//! any particular printed diagram cell-for-cell.

use crate::point::Point;
use crate::stone::Player;

/// The action a game expects next while an opening protocol is in progress.
///
/// Obtained from [`Game::opening_action`](crate::Game::opening_action). Each
/// variant names the method used to satisfy it.
///
/// # Examples
///
/// ```
/// use gomoku::{Game, OpeningAction, RuleSet};
///
/// // A free opening never demands a special action.
/// assert_eq!(Game::new(RuleSet::standard()).opening_action(), OpeningAction::None);
///
/// // A Swap2 game opens by asking for a stone placement.
/// let swap2 = Game::new(RuleSet::swap2());
/// assert!(matches!(swap2.opening_action(), OpeningAction::PlaceStone { .. }));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpeningAction {
    /// Play a stone of `color` (via [`Game::play`](crate::Game::play)); it must
    /// satisfy `constraint`.
    PlaceStone {
        /// The color of the stone to place.
        color: Player,
        /// Where the stone may be placed.
        constraint: Constraint,
    },
    /// Choose which color to continue as
    /// ([`Game::choose_color`](crate::Game::choose_color)).
    ChooseColor,
    /// Make the Swap2 three-way decision
    /// ([`Game::swap2_decision`](crate::Game::swap2_decision)).
    Swap2Decision,
    /// Announce the number of 5th-move candidates
    /// ([`Game::announce_fifth_count`](crate::Game::announce_fifth_count)).
    AnnounceCount,
    /// Offer `count` candidate 5th moves
    /// ([`Game::propose_fifths`](crate::Game::propose_fifths)).
    ProposeFifths {
        /// How many distinct candidate moves must be offered.
        count: u8,
    },
    /// Select one of the offered 5th moves
    /// ([`Game::choose_fifth`](crate::Game::choose_fifth)).
    SelectFifth {
        /// The candidate moves previously proposed.
        options: Vec<Point>,
    },
    /// No opening action pending; play proceeds normally.
    None,
}

/// A placement restriction on an opening move, carried by
/// [`OpeningAction::PlaceStone`].
///
/// # Examples
///
/// ```
/// use gomoku::{Constraint, Point};
///
/// let center = Point::new(7, 7);
/// // The third Pro stone must be at least 3 lines from the center.
/// let constraint = Constraint::MinDistance(3);
/// assert!(constraint.allows(Point::new(10, 7), center));  // distance 3
/// assert!(!constraint.allows(Point::new(9, 7), center));  // distance 2
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint {
    /// May be played on any empty point.
    Anywhere,
    /// Must be the center intersection.
    Center,
    /// Must be at least this Chebyshev distance from the center.
    MinDistance(u8),
}

impl Constraint {
    /// Whether `p` satisfies this constraint on a board with the given `center`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{Constraint, Point};
    ///
    /// let center = Point::new(7, 7);
    /// assert!(Constraint::Anywhere.allows(Point::new(0, 0), center));
    /// assert!(Constraint::Center.allows(center, center));
    /// assert!(!Constraint::Center.allows(Point::new(8, 7), center));
    /// ```
    pub fn allows(self, p: Point, center: Point) -> bool {
        match self {
            Constraint::Anywhere => true,
            Constraint::Center => p == center,
            Constraint::MinDistance(d) => {
                let dx = (p.x as i16 - center.x as i16).unsigned_abs();
                let dy = (p.y as i16 - center.y as i16).unsigned_abs();
                dx.max(dy) >= d as u16
            }
        }
    }
}

/// Player 2's choice in the Swap2 opening, passed to
/// [`Game::swap2_decision`](crate::Game::swap2_decision).
///
/// # Examples
///
/// ```
/// use gomoku::{Game, Point, RuleSet, Swap2Choice};
///
/// let mut game = Game::new(RuleSet::swap2());
/// for &(x, y) in &[(7, 7), (7, 8), (8, 7)] {
///     game.play(Point::new(x, y))?;
/// }
/// // Take Black instead of continuing as White.
/// game.swap2_decision(Swap2Choice::SwapToBlack)?;
/// # Ok::<(), gomoku::MoveError>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Swap2Choice {
    /// Take White and continue (no swap).
    PlayWhite,
    /// Take Black (swap).
    SwapToBlack,
    /// Place two more stones (White then Black) and pass the color choice back.
    PlaceTwoMore,
}

/// One of the 26 canonical Renju openings. The first 13 are *direct* (直), the
/// last 13 are *indirect* (間).
///
/// Each name maps to a concrete three-stone placement via [`placements`], with
/// the reverse lookup provided by [`identify`]. Enumerate them all with
/// [`ALL_OPENINGS`].
///
/// [`placements`]: OpeningName::placements
/// [`identify`]: OpeningName::identify
///
/// # Examples
///
/// ```
/// use gomoku::OpeningName;
///
/// let opening = OpeningName::Kansei;
/// assert_eq!(opening.romaji(), "Kansei");
/// assert!(opening.is_direct());
///
/// // A name round-trips through its board placement.
/// let stones = opening.placements(15);
/// assert_eq!(OpeningName::identify(stones, 15), Some(opening));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpeningName {
    /// Direct opening - 寒星 (Kansei).
    Kansei,
    /// Direct opening - 渓月 (Keigetsu).
    Keigetsu,
    /// Direct opening - 疏星 (Sosei).
    Sosei,
    /// Direct opening - 花月 (Kagetsu).
    Kagetsu,
    /// Direct opening - 残月 (Zangetsu).
    Zangetsu,
    /// Direct opening - 雨月 (Ugetsu).
    Ugetsu,
    /// Direct opening - 金星 (Kinsei).
    Kinsei,
    /// Direct opening - 松月 (Shougetsu).
    Shougetsu,
    /// Direct opening - 丘月 (Kyuugetsu).
    Kyuugetsu,
    /// Direct opening - 新月 (Shingetsu).
    Shingetsu,
    /// Direct opening - 瑞星 (Zuisei).
    Zuisei,
    /// Direct opening - 山月 (Sangetsu).
    Sangetsu,
    /// Direct opening - 遊星 (Yuusei).
    Yuusei,
    /// Indirect opening - 長星 (Chousei).
    Chousei,
    /// Indirect opening - 峡月 (Kyougetsu).
    Kyougetsu,
    /// Indirect opening - 恒星 (Kousei).
    Kousei,
    /// Indirect opening - 水月 (Suigetsu).
    Suigetsu,
    /// Indirect opening - 流星 (Ryuusei).
    Ryuusei,
    /// Indirect opening - 雲月 (Ungetsu).
    Ungetsu,
    /// Indirect opening - 浦月 (Hogetsu).
    Hogetsu,
    /// Indirect opening - 嵐月 (Rangetsu).
    Rangetsu,
    /// Indirect opening - 銀月 (Gingetsu).
    Gingetsu,
    /// Indirect opening - 明星 (Myoujou).
    Myoujou,
    /// Indirect opening - 斜月 (Shagetsu).
    Shagetsu,
    /// Indirect opening - 名月 (Meigetsu).
    Meigetsu,
    /// Indirect opening - 彗星 (Suisei).
    Suisei,
}

/// All 26 openings, direct first then indirect.
///
/// # Examples
///
/// ```
/// use gomoku::{OpeningName, ALL_OPENINGS};
///
/// assert_eq!(ALL_OPENINGS.len(), 26);
/// assert_eq!(ALL_OPENINGS[0], OpeningName::Kansei);
///
/// // Exactly the first half are direct openings.
/// assert_eq!(ALL_OPENINGS.iter().filter(|o| o.is_direct()).count(), 13);
/// ```
pub const ALL_OPENINGS: [OpeningName; 26] = {
    use OpeningName::*;
    [
        Kansei, Keigetsu, Sosei, Kagetsu, Zangetsu, Ugetsu, Kinsei, Shougetsu, Kyuugetsu,
        Shingetsu, Zuisei, Sangetsu, Yuusei, Chousei, Kyougetsu, Kousei, Suigetsu, Ryuusei,
        Ungetsu, Hogetsu, Rangetsu, Gingetsu, Myoujou, Shagetsu, Meigetsu, Suisei,
    ]
};

/// Romaji names, indexed by `OpeningName as usize`.
const ROMAJI: [&str; 26] = [
    "Kansei",
    "Keigetsu",
    "Sosei",
    "Kagetsu",
    "Zangetsu",
    "Ugetsu",
    "Kinsei",
    "Shougetsu",
    "Kyuugetsu",
    "Shingetsu",
    "Zuisei",
    "Sangetsu",
    "Yuusei",
    "Chousei",
    "Kyougetsu",
    "Kousei",
    "Suigetsu",
    "Ryuusei",
    "Ungetsu",
    "Hogetsu",
    "Rangetsu",
    "Gingetsu",
    "Myoujou",
    "Shagetsu",
    "Meigetsu",
    "Suisei",
];

/// Japanese (kanji) names, indexed by `OpeningName as usize`.
const KANJI: [&str; 26] = [
    "寒星", "渓月", "疏星", "花月", "残月", "雨月", "金星", "松月", "丘月", "新月", "瑞星", "山月",
    "遊星", "長星", "峡月", "恒星", "水月", "流星", "雲月", "浦月", "嵐月", "銀月", "明星", "斜月",
    "名月", "彗星",
];

/// Stone-3 offsets (relative to center) for the 13 direct openings.
const DIRECT_S3: [(i8, i8); 13] = [
    (-2, 0),
    (-1, 0),
    (2, 0),
    (-2, 1),
    (-1, 1),
    (0, 1),
    (1, 1),
    (2, 1),
    (-2, 2),
    (-1, 2),
    (0, 2),
    (1, 2),
    (2, 2),
];

/// Stone-3 offsets (relative to center) for the 13 indirect openings.
const INDIRECT_S3: [(i8, i8); 13] = [
    (-2, -2),
    (-2, -1),
    (-1, -1),
    (-2, 0),
    (-1, 0),
    (-2, 1),
    (-1, 1),
    (0, 1),
    (-2, 2),
    (-1, 2),
    (0, 2),
    (1, 2),
    (2, 2),
];

impl OpeningName {
    /// Whether this is a direct (直) opening (stone 2 orthogonally adjacent).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::OpeningName;
    ///
    /// assert!(OpeningName::Kansei.is_direct());    // 直 (direct)
    /// assert!(!OpeningName::Chousei.is_direct());  // 間 (indirect)
    /// ```
    #[inline]
    pub fn is_direct(self) -> bool {
        (self as usize) < 13
    }

    /// The romaji name, e.g. `"Kansei"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::OpeningName;
    ///
    /// assert_eq!(OpeningName::Kagetsu.romaji(), "Kagetsu");
    /// ```
    #[inline]
    pub fn romaji(self) -> &'static str {
        ROMAJI[self as usize]
    }

    /// The Japanese (kanji) name, e.g. `"寒星"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::OpeningName;
    ///
    /// assert_eq!(OpeningName::Kansei.kanji(), "寒星");
    /// ```
    #[inline]
    pub fn kanji(self) -> &'static str {
        KANJI[self as usize]
    }

    /// Offsets of the three opening stones relative to the board center, in
    /// move order (Black, White, Black).
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::OpeningName;
    ///
    /// // Stone 1 is always the center; for a direct opening stone 2 is
    /// // orthogonally adjacent.
    /// let offsets = OpeningName::Kansei.offsets();
    /// assert_eq!(offsets[0], (0, 0));
    /// assert_eq!(offsets[1], (1, 0));
    /// ```
    pub fn offsets(self) -> [(i8, i8); 3] {
        let i = self as usize;
        if self.is_direct() {
            [(0, 0), (1, 0), DIRECT_S3[i]]
        } else {
            [(0, 0), (1, 1), INDIRECT_S3[i - 13]]
        }
    }

    /// The three opening stones as absolute points on a board of `size`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{OpeningName, Point};
    ///
    /// // On a 15×15 board the first stone lands on the center, h8.
    /// let stones = OpeningName::Kansei.placements(15);
    /// assert_eq!(stones[0], Point::new(7, 7));
    /// ```
    pub fn placements(self, size: u8) -> [Point; 3] {
        let c = (size / 2) as i8;
        self.offsets()
            .map(|(dx, dy)| Point::new((c + dx) as u8, (c + dy) as u8))
    }

    /// Identify which named opening `stones` (in move order: Black, White,
    /// Black) represent on a board of `size`, considering all 8 board
    /// symmetries. Returns `None` if they match no canonical opening.
    ///
    /// # Examples
    ///
    /// ```
    /// use gomoku::{OpeningName, Point};
    ///
    /// // The reverse of `placements`: recover the name from the stones.
    /// let stones = OpeningName::Sosei.placements(15);
    /// assert_eq!(OpeningName::identify(stones, 15), Some(OpeningName::Sosei));
    ///
    /// // Three stones in a line are not a canonical opening.
    /// let line = [Point::new(0, 0), Point::new(1, 0), Point::new(2, 0)];
    /// assert_eq!(OpeningName::identify(line, 15), None);
    /// ```
    pub fn identify(stones: [Point; 3], size: u8) -> Option<OpeningName> {
        let c = (size / 2) as i16;
        let off = |p: Point| ((p.x as i16 - c) as i8, (p.y as i16 - c) as i8);
        let want = [off(stones[0]), off(stones[1]), off(stones[2])];

        for name in ALL_OPENINGS {
            let base = name.offsets();
            for t in 0..8 {
                if (0..3).all(|k| transform(base[k], t) == want[k]) {
                    return Some(name);
                }
            }
        }
        None
    }
}

/// Apply one of the 8 dihedral symmetries (rotations + reflections) to an offset.
fn transform((x, y): (i8, i8), t: u8) -> (i8, i8) {
    match t {
        0 => (x, y),
        1 => (-x, y),
        2 => (x, -y),
        3 => (-x, -y),
        4 => (y, x),
        5 => (-y, x),
        6 => (y, -x),
        _ => (-y, -x),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_are_26_distinct_openings() {
        assert_eq!(ALL_OPENINGS.len(), 26);
        let mut seen = std::collections::HashSet::new();
        for name in ALL_OPENINGS {
            // Distinct by (stone2, stone3) offset pair.
            let o = name.offsets();
            assert!(
                seen.insert((o[1], o[2])),
                "duplicate opening {}",
                name.romaji()
            );
        }
        assert_eq!(seen.len(), 26);
        assert_eq!(ALL_OPENINGS.iter().filter(|n| n.is_direct()).count(), 13);
    }

    #[test]
    fn placements_are_on_board_and_distinct() {
        for name in ALL_OPENINGS {
            let pts = name.placements(15);
            assert_ne!(pts[0], pts[1]);
            assert_ne!(pts[0], pts[2]);
            assert_ne!(pts[1], pts[2]);
            for p in pts {
                assert!(p.x < 15 && p.y < 15);
            }
            assert_eq!(pts[0], Point::new(7, 7)); // stone 1 is always center
        }
    }

    #[test]
    fn identify_round_trips() {
        for name in ALL_OPENINGS {
            let pts = name.placements(15);
            assert_eq!(
                OpeningName::identify(pts, 15),
                Some(name),
                "{}",
                name.romaji()
            );
        }
    }

    #[test]
    fn constraint_distance() {
        let center = Point::new(7, 7);
        assert!(Constraint::Center.allows(center, center));
        assert!(!Constraint::Center.allows(Point::new(8, 7), center));
        assert!(!Constraint::MinDistance(3).allows(Point::new(9, 7), center)); // dist 2
        assert!(Constraint::MinDistance(3).allows(Point::new(10, 7), center)); // dist 3
    }
}
