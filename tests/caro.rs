//! Caro rules: a winning row must not be blocked by the opponent at both ends.

use gomoku::{Game, Player, Point, RuleSet, Status};

fn play_all(g: &mut Game, points: &[(u8, u8)]) {
    for &(x, y) in points {
        g.play(Point::new(x, y)).expect("legal move");
    }
}

#[test]
fn five_blocked_both_ends_does_not_win() {
    let mut g = Game::new(RuleSet::caro());
    // White blockers at x=0 and x=6; Black fills x=1..=5 on row 7.
    play_all(
        &mut g,
        &[
            (1, 7),
            (0, 7),
            (2, 7),
            (6, 7),
            (3, 7),
            (0, 0),
            (4, 7),
            (1, 0),
        ],
    );
    let out = g.play(Point::new(5, 7)).unwrap();
    assert_eq!(out.status, Status::InProgress);
}

#[test]
fn five_blocked_one_end_wins() {
    let mut g = Game::new(RuleSet::caro());
    // Only the left end (x=0) is blocked; the right end (x=6) is open.
    play_all(
        &mut g,
        &[
            (1, 7),
            (0, 7),
            (2, 7),
            (0, 0),
            (3, 7),
            (1, 0),
            (4, 7),
            (2, 0),
        ],
    );
    let out = g.play(Point::new(5, 7)).unwrap();
    assert_eq!(out.status, Status::Win(Player::Black));
}

#[test]
fn overline_wins_when_unblocked() {
    let mut g = Game::new(RuleSet::caro());
    // Bridge x=1,2,3 and 5,6 with x=4 to make an open six.
    play_all(
        &mut g,
        &[
            (1, 7),
            (0, 0),
            (2, 7),
            (2, 0),
            (3, 7),
            (4, 0),
            (5, 7),
            (6, 0),
            (6, 7),
            (8, 0),
        ],
    );
    let out = g.play(Point::new(4, 7)).unwrap();
    assert_eq!(out.status, Status::Win(Player::Black));
}
