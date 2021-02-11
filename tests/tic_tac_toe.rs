#[macro_use]
extern crate enum_map;
#[macro_use]
extern crate itertools;
use serde_json::json;

use lib_table_top::games::tic_tac_toe::{
    Col, Col::*, Error::*, GameState, Player, Player::*, Position, Row, Row::*, Status,
    POSSIBLE_WINS,
};

#[test]
fn test_new() {
    let game_state = GameState::new();
    let board = game_state.board();

    for &col in &Col::ALL {
        for &row in &Row::ALL {
            assert_eq!(board[col][row], None);
        }
    }

    let expected: Vec<(Col, Row)> = iproduct!(&Col::ALL, &Row::ALL)
        .map(|(&col, &row)| (col, row))
        .collect();

    assert_eq!(game_state.available().collect::<Vec<Position>>(), expected)
}

#[test]
fn test_make_move() {
    let game_state = GameState::new();
    assert_eq!(game_state.whose_turn(), X);
    let result = game_state.make_move((X, (Col1, Row1)));
    assert!(result.is_ok());
    let game_state = result.unwrap();
    assert_eq!(
        game_state.history().collect::<Vec<(Player, Position)>>(),
        vec![(X, (Col1, Row1))]
    );

    assert_eq!(game_state.whose_turn(), O);

    let pos = (Col1, Row1);
    assert_eq!(
        game_state.make_move((O, (Col1, Row1))),
        Err(SpaceIsTaken { attempted: pos })
    );
    assert_eq!(
        game_state.make_move((X, (Col1, Row2))),
        Err(OtherPlayerTurn { attempted: X })
    );

    let result = game_state.make_move((O, (Col2, Row2)));
    assert!(result.is_ok());
    let game_state = result.unwrap();
    assert_eq!(
        game_state.history().collect::<Vec<(Player, Position)>>(),
        vec![(X, (Col1, Row1)), (O, (Col2, Row2))]
    );
}

#[test]
fn test_undoing_moves() {
    let game_state = GameState::new();
    assert_eq!(game_state.whose_turn(), X);
    let game_state = game_state.make_move((X, (Col1, Row1))).unwrap();
    assert_eq!(
        game_state.history().collect::<Vec<(Player, Position)>>(),
        vec![(X, (Col1, Row1))]
    );

    assert_eq!(game_state.whose_turn(), O);

    // undo a made move
    let (game_state, action) = game_state.undo();
    assert_eq!(action, Some((X, (Col1, Row1))));
    assert_eq!(game_state.whose_turn(), X);
    assert_eq!(
        game_state.history().collect::<Vec<(Player, Position)>>(),
        vec![],
    );
}

#[test]
fn test_you_cant_go_to_the_same_square_twice() {
    let position = (Col1, Row1);
    let game = GameState::new().make_move((X, position)).unwrap();
    let result = game.make_move((O, position));
    assert_eq!(
        result,
        Err(SpaceIsTaken {
            attempted: position
        })
    );
}

#[test]
fn test_you_cant_go_twice_in_a_row() {
    let game = GameState::new();
    assert_eq!(game.whose_turn(), X);
    let game = game.make_move((X, (Col1, Row1))).unwrap();
    assert_eq!(game.whose_turn(), O);
    let result = game.make_move((X, (Col0, Row0)));
    assert_eq!(result, Err(OtherPlayerTurn { attempted: X }));
}

#[test]
fn test_you_can_get_the_board() {
    let game = GameState::new();
    assert_eq!(game.board(), enum_map! { _ => enum_map! { _ => None } });
    let game = game.make_move((X, (Col1, Row1))).unwrap();
    assert_eq!(
        game.board(),
        enum_map! {
            Col1 => enum_map! {
                Row1 => Some(X),
                _ => None
            },
            _ => enum_map! {
                _ => None
            }
        }
    );
    let game = game.make_move((O, (Col1, Row0))).unwrap();
    assert_eq!(game.board()[Col1][Row0], Some(O));
}

#[test]
fn test_you_can_play_and_draw() {
    let game = [
        (X, (Col0, Row0)),
        (O, (Col1, Row0)),
        (X, (Col2, Row0)),
        (O, (Col2, Row1)),
        (X, (Col0, Row1)),
        (O, (Col2, Row2)),
        (X, (Col1, Row1)),
        (O, (Col0, Row2)),
        (X, (Col1, Row2)),
    ]
    .iter()
    .fold(GameState::new(), |game, &action| {
        game.make_move(action).unwrap()
    });

    assert_eq!(game.status(), Status::Draw);
}

#[test]
fn test_you_can_play_and_win() {
    let game = GameState::new();
    assert_eq!(game.status(), Status::InProgress);

    let game = [
        (X, (Col0, Row0)),
        (O, (Col1, Row0)),
        (X, (Col0, Row1)),
        (O, (Col1, Row1)),
    ]
    .iter()
    .fold(game, |game, &action| {
        let game = game.make_move(action).unwrap();
        assert_eq!(game.status(), Status::InProgress);
        game
    });

    let game = game.make_move((X, (Col0, Row2))).unwrap();
    assert_eq!(
        game.status(),
        Status::Win {
            player: X,
            positions: [(Col0, Row0), (Col0, Row1), (Col0, Row2)]
        }
    );
}

#[test]
fn test_try_all_the_potential_wins() {
    for &win in &POSSIBLE_WINS {
        let game = GameState::new();
        let loss: Vec<Position> = game
            .available()
            .filter(|position| !win.contains(position))
            .take(2)
            .map(|position| position.to_owned())
            .collect();

        let game = [
            (X, win[0]),
            (O, loss[0]),
            (X, win[1]),
            (O, loss[1]),
            (X, win[2]),
        ]
        .iter()
        .fold(game, |game, &action| {
            let result = game.make_move(action);
            assert!(result.is_ok());
            result.unwrap()
        });

        assert_eq!(
            game.status(),
            Status::Win {
                player: X,
                positions: win
            }
        );
    }
}

#[test]
fn test_serializing_tic_tac_toe() {
    let game: GameState = Default::default();

    let serialized = serde_json::to_value(&game).unwrap();
    assert_eq!(serialized, json!({ "history": [] }));

    let deserialized: GameState = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized, game);

    let game = game.make_move((X, (Col1, Row1))).unwrap();

    let serialized = serde_json::to_value(&game).unwrap();
    assert_eq!(serialized, json!({ "history": [[1, 1]] }));

    let deserialized: GameState = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized, game);

    let game = game.make_move((O, (Col2, Row2))).unwrap();

    let serialized = serde_json::to_value(&game).unwrap();
    assert_eq!(serialized, json!({ "history": [[1, 1], [2, 2]] }));

    let deserialized: GameState = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized, game);
}
