use lib_table_top::games::tic_tac_toe::{
    Col::*, GameState, Marker::*, Position, Row::*, Status, POSSIBLE_WINS,
};

#[test]
fn test_you_can_play_and_win() {
    let mut game = GameState::new();
    assert_eq!(game.status(), Status::InProgress);

    let moves = [
        (X, (Col0, Row0)),
        (O, (Col1, Row0)),
        (X, (Col0, Row1)),
        (O, (Col1, Row1)),
    ];

    for &(marker, position) in &moves {
        let result = game.make_move(marker, position);
        assert!(result.is_ok());
        assert_eq!(game.status(), Status::InProgress);
    }

    let result = game.make_move(X, (Col0, Row2));
    assert!(result.is_ok());
    assert_eq!(
        game.status(),
        Status::Win {
            marker: X,
            spaces: [(Col0, Row0), (Col0, Row1), (Col0, Row2)]
        }
    );
}

#[test]
fn test_try_all_the_potential_wins() {
    for &win in &POSSIBLE_WINS {
        let mut game = GameState::new();
        let loss: Vec<Position> = game
            .available()
            .iter()
            .filter(|position| !win.contains(position))
            .take(2)
            .map(|position| position.to_owned())
            .collect();

        let results = vec![
            game.make_move(X, win[0]),
            game.make_move(O, loss[0]),
            game.make_move(X, win[1]),
            game.make_move(O, loss[1]),
            game.make_move(X, win[2]),
        ];

        for result in &results {
            assert!(result.is_ok());
        }

        assert_eq!(
            game.status(),
            Status::Win {
                marker: X,
                spaces: win
            }
        );
    }
}
