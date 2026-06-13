//! Omok: 19×19 board, exactly five wins (overline does not).

use gomoku::{Game, Player, Point, RuleSet, Status};

#[test]
fn omok_is_19x19() {
    let g = Game::new(RuleSet::omok());
    assert_eq!(g.board().size(), 19);
}

#[test]
fn exact_five_wins_on_omok_board() {
    let mut g = Game::new(RuleSet::omok());
    let blacks = [(2, 2), (3, 2), (4, 2), (5, 2), (6, 2)];
    let whites = [(2, 10), (4, 10), (6, 10), (8, 10)];
    for i in 0..4 {
        g.play(Point::new(blacks[i].0, blacks[i].1)).unwrap();
        g.play(Point::new(whites[i].0, whites[i].1)).unwrap();
    }
    let out = g.play(Point::new(6, 2)).unwrap();
    assert_eq!(out.status, Status::Win(Player::Black));
}
