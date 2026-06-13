//! Compile-time guarantees that the public types are thread-transferable.
//!
//! These functions don't run any logic; they fail to *compile* if any of the
//! listed types stops being `Send + Sync` (for example, if interior mutability
//! or an `Rc` were introduced). That makes the crate's concurrency contract
//! part of the test suite.

use gomoku::{
    Board, Cell, Constraint, ForbiddenKind, Game, MoveError, MoveOutcome, Opening, OpeningAction,
    OpeningName, Player, Point, RuleSet, Status, Swap2Choice,
};

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}
fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn public_types_are_send_and_sync() {
    assert_send_sync::<Game>();
    assert_send_sync::<Board>();
    assert_send_sync::<RuleSet>();
    assert_send_sync::<Point>();
    assert_send_sync::<Player>();
    assert_send_sync::<Cell>();
    assert_send_sync::<Status>();
    assert_send_sync::<MoveOutcome>();
    assert_send_sync::<MoveError>();
    assert_send_sync::<ForbiddenKind>();
    assert_send_sync::<Opening>();
    assert_send_sync::<OpeningAction>();
    assert_send_sync::<OpeningName>();
    assert_send_sync::<Constraint>();
    assert_send_sync::<Swap2Choice>();

    // Sanity: `Send` and `Sync` individually, too.
    assert_send::<Game>();
    assert_sync::<Game>();
}
