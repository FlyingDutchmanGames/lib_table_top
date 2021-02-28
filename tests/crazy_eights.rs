use lib_table_top::common::rand::RngSeed;
use lib_table_top::games::crazy_eights::{
    GameHistory, GameState, NumberOfPlayers, PlayerView, Settings,
};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_serializing_crazy_eights_player_view() {
    let settings = Settings {
        seed: RngSeed([0; 32]),
        number_of_players: NumberOfPlayers::Three,
    };
    let game = GameState::new(Arc::new(settings));

    let action = game.current_player_view().valid_actions().pop().unwrap();
    let player = game.whose_turn();
    let game = game.apply_action((player, action)).unwrap();

    let action = game.current_player_view().valid_actions().pop().unwrap();
    let player = game.whose_turn();
    let game = game.apply_action((player, action)).unwrap();

    let expected = json!({
        "observer_view": {
            "whose_turn": "P3",
            "current_suit": "Spades",
            "top_card": [8, "Hearts"],
            "discarded": [[4, "Diamonds"], [11, "Diamonds"]],
            "draw_pile_remaining": 36,
            "player_card_count": {
                "P1": 4,
                "P2": 4,
                "P3": 5,
            }
        },
        "player": "P3",
        "hand": [
            [7, "Clubs"],
            [10, "Hearts"],
            [2, "Diamonds"],
            [9, "Clubs"],
            [12, "Clubs"],
        ],
    });

    let serialized = serde_json::to_value(game.current_player_view()).unwrap();
    assert_eq!(serialized, expected);

    // Def couldn't figure out how to go between PlayerView<Vec<Card>> to PlayerView<&[Card]> so
    // just test deserializing it again
    let deserialized: PlayerView = serde_json::from_value(serialized).unwrap();
    assert_eq!(serde_json::to_value(deserialized).unwrap(), expected);
}

#[test]
fn test_serializing_and_deserializing_crazy_eights_game_history() {
    let settings = Settings {
        seed: RngSeed([0; 32]),
        number_of_players: NumberOfPlayers::Three,
    };
    let game = GameState::new(Arc::new(settings));

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
    let game = game.apply_action((player, action)).unwrap();

    let action = game.current_player_view().valid_actions().pop().unwrap();
    let player = game.whose_turn();
    let game = game.apply_action((player, action)).unwrap();

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

    let deserialized: GameHistory = serde_json::from_value(serialized).unwrap();
    assert_eq!(&deserialized, game.game_history());
}
