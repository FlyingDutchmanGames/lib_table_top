use itertools::iterate;
use lib_table_top::games::marooned::{
    Action, Col, Dimensions, GameState, Player::*, Position, Row, Settings, SettingsBuilder,
    SettingsError::*, Status::*,
};
use serde_json::json;

#[test]
fn test_dimensions() {
    let dimensions: Dimensions = Default::default();

    for &(col, row) in [(0, 100), (100, 0)].iter() {
        assert!(!dimensions.is_position_on_board((Col(col), Row(row))))
    }
    for &(col, row) in [(0, 0), (1, 1), (5, 7)].iter() {
        assert!(dimensions.is_position_on_board((Col(col), Row(row))))
    }

    let dimensions = Dimensions::new(2, 3).unwrap();
    assert_eq!(
        dimensions.all_positions().collect::<Vec<Position>>(),
        vec![
            (Col(0), Row(0)),
            (Col(0), Row(1)),
            (Col(1), Row(0)),
            (Col(1), Row(1)),
            (Col(2), Row(0)),
            (Col(2), Row(1))
        ]
    );
}

#[test]
fn test_making_a_few_moves() {
    let game = GameState::new(Default::default());
    assert_eq!(game.status(), InProgress);
    assert_eq!(game.whose_turn(), P1);
    assert_eq!(game.removed().next(), None);

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

    let position_to_remove = game.removable().next().unwrap();
    let move_to = allowed_movements.first().unwrap().to_owned();
    let game = game
        .apply_action(Action {
            player: P1,
            remove: position_to_remove,
            to: move_to,
        })
        .unwrap();

    assert_eq!(game.player_position(P1), move_to);
    assert_eq!(game.whose_turn(), P2);
    assert_eq!(game.removed().next(), Some(position_to_remove));
}

#[test]
fn test_make_a_new_game_from_settings_builder() {
    let game = SettingsBuilder::new()
        .rows(10)
        .cols(9)
        .p1_starting((Col(0), Row(0)))
        .p2_starting((Col(1), Row(1)))
        .starting_removed(vec![(Col(2), Row(2))])
        .build_game()
        .unwrap();

    assert_eq!(game.player_position(P1), (Col(0), Row(0)));
    assert_eq!(game.player_position(P2), (Col(1), Row(1)));
    assert_eq!(game.dimensions().rows, 10);
    assert_eq!(game.dimensions().cols, 9);
}

#[test]
fn test_a_full_game() {
    let game = SettingsBuilder::new().rows(3).cols(3).build_game().unwrap();

    let _ = iterate(game, |game| {
        // all valid actions are valid!
        for action in game.valid_actions() {
            assert!(game.apply_action(action).is_ok());
        }

        match game.valid_actions().next() {
            Some(action) => game.apply_action(action).unwrap(),
            None => game.clone(),
        }
    })
    .inspect(|game| match game.status() {
        InProgress => {
            let target = game
                .allowed_movement_targets_for_player(game.whose_turn())
                .next();
            assert!(target != None);
        }
        Win { player } => {
            assert_eq!(player, game.whose_turn().opponent());
            assert_eq!(
                game.allowed_movement_targets_for_player(game.whose_turn())
                    .collect::<Vec<Position>>(),
                vec![]
            );
        }
    })
    .take_while(|game| game.status() == InProgress)
    .collect::<Vec<GameState>>();
}

#[test]
fn test_settings_handle_invalid_inputs() {
    for &(rows, cols) in &[(0, 0), (0, 2), (2, 0), (1, 1)] {
        assert_eq!(Err(InvalidDimensions), Dimensions::new(rows, cols));
    }
}

#[test]
fn test_serializing_dimensions() {
    let dimensions: Dimensions = Default::default();
    let serialized = serde_json::to_value(&dimensions).unwrap();
    assert_eq!(serialized, json!({"rows": 8, "cols": 6}));
    let deserialized: Dimensions = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized, dimensions);
}

#[test]
fn test_serializing_actions() {
    let action = Action {
        to: (Col(0), Row(0)),
        remove: (Col(1), Row(1)),
        player: P1,
    };
    let serialized = serde_json::to_value(&action).unwrap();
    assert_eq!(
        serialized,
        json!({"player": 1, "to": [0, 0], "remove": [1, 1]})
    );
    let deserialized: Action = serde_json::from_value(serialized).unwrap();
    assert_eq!(action, deserialized);
}

#[test]
fn test_serializing_settings() {
    let settings = SettingsBuilder::new()
        .starting_removed(vec![(Col(0), Row(0))])
        .build()
        .unwrap();
    let serialized = serde_json::to_value(&settings).unwrap();
    assert_eq!(
        serialized,
        json!({
            "dimensions": {"cols": 6, "rows": 8},
            "p1_starting": [3, 0],
            "p2_starting": [2, 7],
            "starting_removed": [[0, 0]],
        })
    );
    let deserialized: Settings = serde_json::from_value(serialized).unwrap();
    assert_eq!(settings, deserialized);
}

#[test]
fn test_serializing_game_state() {
    let game: GameState = SettingsBuilder::new()
        .starting_removed(vec![(Col(0), Row(0))])
        .build_game()
        .unwrap();

    let (game, actions) = (1..=3).fold((game, vec![]), |(game, mut actions), _| {
        let action = game.valid_actions().next().unwrap();
        let game = game.apply_action(action).unwrap();
        actions.push(action);
        (game, actions)
    });

    let serialized = serde_json::to_value(&game).unwrap();
    assert_eq!(
        serialized,
        json!({
            "history": actions,
            "settings": {
                "dimensions": {
                    "cols": 6, "rows": 8
                },
                "p1_starting": [3, 0],
                "p2_starting": [2, 7],
                "starting_removed": [[0, 0]]
            },
        })
    );
    let deserialized: GameState = serde_json::from_value(serialized).unwrap();
    assert_eq!(game, deserialized);
}
