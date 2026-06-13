//! Integration tests exercising the public API under Standard rules.

use gomoku::{Game, Player, Point, RuleSet, Status};

/// Play `points` alternating starting with Black, asserting each is legal.
fn play_all(g: &mut Game, points: &[(u8, u8)]) {
    for &(x, y) in points {
        g.play(Point::new(x, y)).expect("legal move");
    }
}

#[test]
fn exact_five_wins() {
    let mut g = Game::new(RuleSet::standard());
    // Interleave Black's row with harmless White moves far away.
    play_all(
        &mut g,
        &[
            (0, 0),
            (0, 5),
            (1, 0),
            (1, 5),
            (2, 0),
            (2, 5),
            (3, 0),
            (3, 5),
        ],
    );
    let out = g.play(Point::new(4, 0)).unwrap();
    assert_eq!(out.status, Status::Win(Player::Black));
}

#[test]
fn overline_does_not_win_in_standard() {
    let mut g = Game::new(RuleSet::standard());
    // Black fills a gap to make six in a row, never passing through an exact
    // five: stones at x = 1,2,3 and 5,6 then the bridge at x = 4.
    play_all(
        &mut g,
        &[
            (1, 0),
            (0, 8),
            (2, 0),
            (2, 8),
            (3, 0),
            (4, 8),
            (5, 0),
            (6, 8),
            (6, 0),
            (8, 8),
        ],
    );
    let out = g.play(Point::new(4, 0)).unwrap(); // bridges into a six
    assert_eq!(out.status, Status::InProgress);
}

#[test]
fn undo_rewinds_a_win() {
    let mut g = Game::new(RuleSet::standard());
    play_all(
        &mut g,
        &[
            (0, 0),
            (0, 5),
            (1, 0),
            (1, 5),
            (2, 0),
            (2, 5),
            (3, 0),
            (3, 5),
        ],
    );
    g.play(Point::new(4, 0)).unwrap();
    assert_eq!(g.status(), Status::Win(Player::Black));
    g.undo();
    assert_eq!(g.status(), Status::InProgress);
    assert_eq!(g.to_move(), Player::Black);
    assert!(g.is_legal(Point::new(4, 0)));
}
