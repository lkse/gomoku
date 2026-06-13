//! Pente: custodial captures, capture counting, capture-win, and undo.

use gomoku::{Game, Player, Point, RuleSet, Status};

#[test]
fn capture_removes_pair_and_counts_it() {
    let mut g = Game::new(RuleSet::pente());
    g.play(Point::new(4, 7)).unwrap(); // black anchor
    g.play(Point::new(5, 7)).unwrap(); // white
    g.play(Point::new(10, 0)).unwrap(); // black elsewhere
    g.play(Point::new(6, 7)).unwrap(); // white (now a pair: 5,6)
                                       // Black flanks the pair by playing (7,7).
    let out = g.play(Point::new(7, 7)).unwrap();
    assert_eq!(out.captured.len(), 2);
    assert_eq!(g.captures(Player::Black), 1);
    assert!(g.board().is_empty(Point::new(5, 7)));
    assert!(g.board().is_empty(Point::new(6, 7)));
}

#[test]
fn undo_restores_captured_stones_and_count() {
    let mut g = Game::new(RuleSet::pente());
    g.play(Point::new(4, 7)).unwrap();
    g.play(Point::new(5, 7)).unwrap();
    g.play(Point::new(10, 0)).unwrap();
    g.play(Point::new(6, 7)).unwrap();
    g.play(Point::new(7, 7)).unwrap(); // capture
    assert_eq!(g.captures(Player::Black), 1);
    g.undo();
    assert_eq!(g.captures(Player::Black), 0);
    assert_eq!(
        g.board().get(Point::new(5, 7)),
        gomoku::Cell::Stone(Player::White)
    );
    assert_eq!(
        g.board().get(Point::new(6, 7)),
        gomoku::Cell::Stone(Player::White)
    );
}

#[test]
fn five_pairs_captured_wins() {
    let mut g = Game::new(RuleSet::pente());
    // Stage five separate capturable White pairs on rows 0..5, each anchored by
    // a Black stone, then let Black flank them one by one.
    for row in 0..5u8 {
        let y = row * 2; // spaced rows so lines never interact
                         // Black anchor at x=4; White pair at x=5,6; Black will flank at x=7.
        g.play(Point::new(4, y)).unwrap(); // black anchor
        g.play(Point::new(5, y)).unwrap(); // white
        g.play(Point::new(12, y)).unwrap(); // black throwaway
        g.play(Point::new(6, y)).unwrap(); // white -> pair formed
        let out = g.play(Point::new(7, y)).unwrap(); // black flanks -> capture
        assert_eq!(out.captured.len(), 2);
        if row < 4 {
            // Not yet five pairs; give White a harmless reply far away.
            assert_eq!(g.status(), Status::InProgress);
            g.play(Point::new(0, y)).unwrap();
        }
    }
    assert_eq!(g.captures(Player::Black), 5);
    assert_eq!(g.status(), Status::Win(Player::Black));
}
