use lib_table_top::games::marooned::{Action, Col, GameState, Player::*, Position, Row, Status::*};

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
