use lib_table_top::common::rand::RngSeed;
use lib_table_top::games::crazy_eights::{GameState, NumberOfPlayers, Settings};
use serde_json::json;

#[test]
fn test_serializing_crazy_eights_game_history() {
    let settings = Settings {
        seed: RngSeed([0; 32]),
        number_of_players: NumberOfPlayers::Three,
    };
    let mut game = GameState::new(settings);

    let serialized = serde_json::to_value(game.game_history()).unwrap();
    assert_eq!(
        serialized,
        json!({
            "settings": {
                "seed": "0000000000000000000000000000000000000000000000000000000000000000",
                "number_of_players": 3,
            },
            "history": []
        })
    );

    let action = game.current_player_view().valid_actions().pop().unwrap();
    let player = game.whose_turn();
    assert!(game.make_move((player, action)).is_ok());

    let action = game.current_player_view().valid_actions().pop().unwrap();
    let player = game.whose_turn();
    assert!(game.make_move((player, action)).is_ok());

    let serialized = serde_json::to_value(game.game_history()).unwrap();
    assert_eq!(
        serialized,
        json!({
            "settings": {
                "seed": "0000000000000000000000000000000000000000000000000000000000000000",
                "number_of_players": 3,
            },
            "history": [
                {"Play": [11, "Diamonds"]},
                {"PlayEight": [[8, "Hearts"], "Spades"]},
            ]
        })
    );
}
