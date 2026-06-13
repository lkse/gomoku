//! A tiny scripted game that prints the board and outcome.
//! Run with `cargo run --example demo`.

use gomoku::{Game, Point, RuleSet};

fn main() {
    let mut game = Game::new(RuleSet::standard());

    // Black makes a horizontal five along row 8 while White answers on row 12.
    let black = [(5, 7), (6, 7), (7, 7), (8, 7), (9, 7)];
    let white = [(5, 11), (6, 11), (7, 11), (8, 11)];

    for (i, &(bx, by)) in black.iter().enumerate() {
        game.play(Point::new(bx, by)).unwrap();
        if let Some(&(wx, wy)) = white.get(i) {
            game.play(Point::new(wx, wy)).unwrap();
        }
    }

    println!("{}\n", game.board());
    println!("status : {:?}", game.status());
    println!("moves  : {}", game.to_move_list());
    println!("fen    : {}", game.board().to_fen());
}
