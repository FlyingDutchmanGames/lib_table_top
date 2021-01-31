use lib_table_top::games::marooned::{
    Action, Col, Dimensions, GameState, Player::*, Position, Row, SettingsError::*, Status::*,
};

#[test]
fn test_making_a_few_moves() {
    let mut game = GameState::new(Default::default());
    assert_eq!(game.status(), InProgress);
    assert_eq!(game.whose_turn(), P1);
    assert_eq!(game.removed_positions().next(), None);

    let allowed_movements: Vec<Position> = game.allowed_movement_targets_for_player(P1).collect();

    assert_eq!(
        allowed_movements,
        vec![
            (Col(3), Row(1)),
            (Col(3), Row(0)),
            (Col(2), Row(1)),
            (Col(1), Row(1)),
            (Col(1), Row(0))
        ]
    );

    assert_eq!(game.player_position(P1), (Col(2), Row(0)));
    assert_eq!(game.player_position(P2), (Col(3), Row(7)));

    let position_to_remove = game.removable_positions().next().unwrap();
    let move_to = allowed_movements.first().unwrap().to_owned();
    assert!(game
        .make_move(Action {
            player: P1,
            remove: position_to_remove,
            to: move_to
        })
        .is_ok());

    assert_eq!(game.player_position(P1), move_to);
    assert_eq!(game.whose_turn(), P2);
    assert_eq!(game.removed_positions().next(), Some(position_to_remove));
}

#[test]
fn test_undoing() {
    let mut game = GameState::new(Default::default());
    let original = game.clone();
    let next_move = game.valid_next_action().unwrap();

    assert_eq!(game.make_move(next_move), Ok(()));
    assert!(original != game);
    assert_eq!(game.undo(), Some(next_move));
    assert!(original == game);
}

#[test]
fn test_a_full_game() {
    let mut game = GameState::new(Default::default());

    loop {
        match game.status() {
            InProgress => {
                let action = game.valid_next_action().unwrap();
                assert!(game.make_move(action).is_ok());
            }
            Win { player } => {
                assert_eq!(player, game.whose_turn().opponent());
                assert_eq!(
                    game.allowed_movement_targets_for_player(game.whose_turn())
                        .collect::<Vec<Position>>(),
                    vec![]
                );
                break;
            }
        }
    }
}

#[test]
fn test_settings_handle_invalid_inputs() {
    for &(rows, cols) in &[(0, 0), (0, 2), (2, 0), (1, 1)] {
        assert_eq!(Err(InvalidDimensions), Dimensions::new(rows, cols));
    }
}
