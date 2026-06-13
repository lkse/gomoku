//! RuleSet validation and the non-panicking `Game::try_new` constructor.

use gomoku::{Capture, Game, RuleSet, RuleSetError};

const PRESETS: &[fn() -> RuleSet] = &[
    RuleSet::freestyle,
    RuleSet::standard,
    RuleSet::renju,
    RuleSet::renju_yamaguchi,
    RuleSet::caro,
    RuleSet::omok,
    RuleSet::pente,
    RuleSet::swap,
    RuleSet::swap2,
    RuleSet::pro,
    RuleSet::long_pro,
];

#[test]
fn every_preset_is_valid() {
    for make in PRESETS {
        let rules = make();
        assert_eq!(rules.validate(), Ok(()));
        assert!(Game::try_new(rules).is_ok());
    }
}

#[test]
fn try_new_rejects_bad_board_size() {
    let mut rules = RuleSet::standard();
    rules.board_size = 4;
    assert_eq!(rules.validate(), Err(RuleSetError::BoardSize(4)));
    assert!(matches!(
        Game::try_new(rules),
        Err(RuleSetError::BoardSize(4))
    ));
}

#[test]
fn try_new_rejects_bad_win_length() {
    let mut rules = RuleSet::standard();
    rules.win_length = 99; // larger than the board
    assert_eq!(rules.validate(), Err(RuleSetError::WinLength(99)));
}

#[test]
fn try_new_rejects_zero_capture_pairs() {
    let mut rules = RuleSet::pente();
    rules.capture = Some(Capture { pairs_to_win: 0 });
    assert_eq!(rules.validate(), Err(RuleSetError::ZeroCapturePairs));
}
