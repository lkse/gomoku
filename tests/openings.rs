//! Opening protocols and the 26-opening catalog, via the public API.

use gomoku::{
    Constraint, Game, MoveError, OpeningAction, OpeningError, OpeningName, Player, Point, RuleSet,
    Status, Swap2Choice, ALL_OPENINGS,
};

#[test]
fn catalog_has_26_and_round_trips() {
    assert_eq!(ALL_OPENINGS.len(), 26);
    assert_eq!(ALL_OPENINGS.iter().filter(|o| o.is_direct()).count(), 13);
    for name in ALL_OPENINGS {
        let pts = name.placements(15);
        assert_eq!(OpeningName::identify(pts, 15), Some(name));
        assert!(!name.romaji().is_empty());
        assert!(!name.kanji().is_empty());
    }
}

#[test]
fn pro_restricts_first_and_third_moves() {
    let mut g = Game::new(RuleSet::pro());
    assert_eq!(
        g.opening_action(),
        OpeningAction::PlaceStone {
            color: Player::Black,
            constraint: Constraint::Center
        }
    );
    // First Black stone must be the center.
    assert_eq!(
        g.play(Point::new(0, 0)),
        Err(MoveError::Opening(OpeningError::PlacementRestricted))
    );
    g.play(Point::new(7, 7)).unwrap(); // center
    g.play(Point::new(7, 8)).unwrap(); // white, anywhere

    // Third stone (Black) must be at least 3 from center.
    assert_eq!(
        g.play(Point::new(8, 8)),
        Err(MoveError::Opening(OpeningError::PlacementRestricted))
    );
    g.play(Point::new(10, 7)).unwrap(); // distance 3 - ok
    assert_eq!(g.opening_action(), OpeningAction::None);
    // Normal play resumes.
    g.play(Point::new(5, 5)).unwrap();
}

#[test]
fn swap_places_three_then_chooses_color() {
    let mut g = Game::new(RuleSet::swap());
    g.play(Point::new(7, 7)).unwrap(); // black
    g.play(Point::new(7, 8)).unwrap(); // white
    g.play(Point::new(8, 7)).unwrap(); // black
    assert_eq!(g.opening_action(), OpeningAction::ChooseColor);
    // A placement is rejected while a decision is pending.
    assert_eq!(
        g.play(Point::new(0, 0)),
        Err(MoveError::Opening(OpeningError::DecisionRequired))
    );
    assert!(g.legal_moves().is_empty());
    g.choose_color(Player::White).unwrap();
    assert_eq!(g.opening_color_choice(), Some(Player::White));
    assert_eq!(g.opening_action(), OpeningAction::None);
    assert_eq!(g.to_move(), Player::White);
    g.play(Point::new(9, 9)).unwrap();
}

#[test]
fn swap2_place_two_more_branch() {
    let mut g = Game::new(RuleSet::swap2());
    g.play(Point::new(7, 7)).unwrap();
    g.play(Point::new(7, 8)).unwrap();
    g.play(Point::new(8, 7)).unwrap();
    assert_eq!(g.opening_action(), OpeningAction::Swap2Decision);
    g.swap2_decision(Swap2Choice::PlaceTwoMore).unwrap();
    // Two more stones: White then Black.
    assert_eq!(g.to_move(), Player::White);
    g.play(Point::new(5, 5)).unwrap();
    assert_eq!(g.to_move(), Player::Black);
    g.play(Point::new(9, 9)).unwrap();
    assert_eq!(g.opening_action(), OpeningAction::ChooseColor);
    g.choose_color(Player::Black).unwrap();
    assert_eq!(g.opening_action(), OpeningAction::None);
    g.play(Point::new(3, 3)).unwrap(); // normal play
}

#[test]
fn swap2_immediate_choice_ends_opening() {
    let mut g = Game::new(RuleSet::swap2());
    g.play(Point::new(7, 7)).unwrap();
    g.play(Point::new(7, 8)).unwrap();
    g.play(Point::new(8, 7)).unwrap();
    g.swap2_decision(Swap2Choice::PlayWhite).unwrap();
    assert_eq!(g.opening_action(), OpeningAction::None);
    assert_eq!(g.to_move(), Player::White);
}

#[test]
fn full_yamaguchi_sequence() {
    let mut g = Game::new(RuleSet::renju_yamaguchi());
    // Opening three stones (Black, White, Black).
    g.play(Point::new(7, 7)).unwrap();
    g.play(Point::new(7, 8)).unwrap();
    g.play(Point::new(8, 7)).unwrap();

    // Announce 2 candidate 5th moves.
    assert_eq!(g.opening_action(), OpeningAction::AnnounceCount);
    assert_eq!(
        g.announce_fifth_count(0),
        Err(MoveError::Opening(OpeningError::ZeroCount))
    );
    g.announce_fifth_count(2).unwrap();

    // Swap decision (no swap here).
    assert_eq!(g.opening_action(), OpeningAction::ChooseColor);
    g.choose_color(Player::Black).unwrap();

    // White plays stone 4.
    assert_eq!(g.to_move(), Player::White);
    g.play(Point::new(5, 5)).unwrap();

    // Black proposes two candidate 5th moves.
    assert_eq!(
        g.opening_action(),
        OpeningAction::ProposeFifths { count: 2 }
    );
    assert_eq!(
        g.propose_fifths(&[Point::new(3, 3)]), // wrong count
        Err(MoveError::Opening(OpeningError::WrongProposalCount))
    );
    g.propose_fifths(&[Point::new(3, 3), Point::new(11, 11)])
        .unwrap();

    // White selects one; it is played as Black.
    match g.opening_action() {
        OpeningAction::SelectFifth { options } => assert_eq!(options.len(), 2),
        other => panic!("expected SelectFifth, got {other:?}"),
    }
    assert_eq!(
        g.choose_fifth(Point::new(9, 9)), // not offered
        Err(MoveError::Opening(OpeningError::NotProposed))
    );
    let out = g.choose_fifth(Point::new(3, 3)).unwrap();
    assert_eq!(out.status, Status::InProgress);

    // Opening complete; normal play, White to move (5 stones placed).
    assert_eq!(g.opening_action(), OpeningAction::None);
    assert_eq!(g.to_move(), Player::White);
    assert_eq!(
        g.board().get(Point::new(3, 3)),
        gomoku::Cell::Stone(Player::Black)
    );
}
