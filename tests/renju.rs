//! Renju: Black's forbidden moves enforced through the public game API.

use gomoku::{ForbiddenKind, Game, MoveError, Player, Point, RuleSet, Status};

/// Drive a game to the position just before Black plays the test move, by
/// placing the listed Black stones with throwaway White replies in between.
fn setup(black: &[(u8, u8)]) -> Game {
    let mut g = Game::new(RuleSet::renju());
    // Park White stones along a far row, spaced so they never line up.
    let mut wx = 0u8;
    for &(x, y) in black {
        g.play(Point::new(x, y)).expect("black setup move");
        g.play(Point::new(wx, 14)).expect("white setup move");
        wx += 2;
    }
    g
}

#[test]
fn black_double_three_is_rejected() {
    let mut g = setup(&[(5, 7), (6, 7), (7, 5)]);
    // Black now to move; (7,7) would make a 3-3 fork (needs (7,6) too).
    g.play(Point::new(7, 6)).unwrap(); // black
    g.play(Point::new(12, 0)).unwrap(); // white elsewhere
    assert_eq!(
        g.play(Point::new(7, 7)),
        Err(MoveError::Forbidden(ForbiddenKind::DoubleThree))
    );
    assert!(!g.is_legal(Point::new(7, 7)));
    assert!(!g.legal_moves().contains(&Point::new(7, 7)));
}

#[test]
fn white_is_not_restricted() {
    // The same 3-3 shape is perfectly legal for White.
    let mut g = Game::new(RuleSet::renju());
    g.play(Point::new(0, 0)).unwrap(); // black throwaway
    g.play(Point::new(5, 7)).unwrap(); // white
    g.play(Point::new(0, 2)).unwrap();
    g.play(Point::new(6, 7)).unwrap(); // white
    g.play(Point::new(0, 4)).unwrap();
    g.play(Point::new(7, 5)).unwrap(); // white
    g.play(Point::new(0, 6)).unwrap();
    g.play(Point::new(7, 6)).unwrap(); // white
    g.play(Point::new(0, 8)).unwrap();
    // White makes the 3-3 fork - allowed.
    assert!(g.play(Point::new(7, 7)).is_ok());
}

#[test]
fn black_five_wins_despite_overline_neighbors() {
    // A move that makes exactly five is always legal and wins.
    let mut g = setup(&[(4, 7), (5, 7), (6, 7), (7, 7)]);
    let out = g.play(Point::new(8, 7)).unwrap();
    assert_eq!(out.status, Status::Win(Player::Black));
}

#[test]
fn black_overline_is_rejected() {
    // Black has X X X X _ X; filling the gap would make six - forbidden.
    let mut g = setup(&[(4, 7), (5, 7), (6, 7), (7, 7), (9, 7)]);
    assert_eq!(
        g.play(Point::new(8, 7)),
        Err(MoveError::Forbidden(ForbiddenKind::Overline))
    );
}
