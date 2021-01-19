use enum_map::EnumMap;
use thiserror::Error;

#[derive(Copy, Clone, Debug, Enum, PartialEq, Eq)]
pub enum Marker {
    X,
    O,
}

use Marker::*;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("space is taken")]
    SpaceIsTaken,
    #[error("not {:?}'s turn", attempted_player)]
    OtherPlayerTurn { attempted_player: Marker },
}

use Error::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum)]
pub enum Row {
    Row0,
    Row1,
    Row2,
}

impl Row {
    const ALL: [Self; 3] = [Row0, Row1, Row2];
}

use Row::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum)]
pub enum Col {
    Col0,
    Col1,
    Col2,
}

impl Col {
    const ALL: [Self; 3] = [Col0, Col1, Col2];
}

use Col::*;

pub struct GameState {
    board: EnumMap<Col, EnumMap<Row, Option<Marker>>>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            board: enum_map! { _ => enum_map! { _ => None } },
        }
    }

    pub fn available(&self) -> Vec<(Col, Row)> {
        iproduct!(&Col::ALL, &Row::ALL)
            .filter(|&(&col, &row)| self.board[col][row].is_none())
            .map(|(&col, &row)| (col, row))
            .collect()
    }

    pub fn whose_turn(&self) -> Marker {
        let mut count: EnumMap<Marker, u8> = enum_map! { _ => 0 };

        self.board
            .iter()
            .flat_map(|(_col_num, row)| row.iter())
            .filter_map(|(_row_num, &marker)| marker)
            .for_each(|marker| count[marker] += 1);

        if count[X] == count[O] {
            X
        } else {
            O
        }
    }

    pub fn make_move(&mut self, marker: Marker, (col, row): (Col, Row)) -> Result<(), Error> {
        if marker != self.whose_turn() {
            return Err(OtherPlayerTurn {
                attempted_player: marker,
            });
        }

        if self.board[col][row] != None {
            return Err(SpaceIsTaken);
        }

        self.board[col][row] = Some(marker);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let game_state = GameState::new();

        for &col in &Col::ALL {
            for &row in &Row::ALL {
                assert_eq!(game_state.board[col][row], None);
            }
        }

        let expected: Vec<(Col, Row)> = iproduct!(&Col::ALL, &Row::ALL)
            .map(|(&col, &row)| (col, row))
            .collect();

        assert_eq!(game_state.available(), expected)
    }

    #[test]
    fn test_make_move() {
        let mut game_state = GameState::new();
        assert_eq!(game_state.whose_turn(), X);
        assert_eq!(game_state.make_move(X, (Col1, Row1)), Ok(()));

        assert_eq!(game_state.whose_turn(), O);

        assert_eq!(game_state.make_move(O, (Col1, Row1)), Err(SpaceIsTaken));
        assert_eq!(
            game_state.make_move(X, (Col1, Row1)),
            Err(OtherPlayerTurn {
                attempted_player: X
            })
        );

        assert_eq!(game_state.make_move(O, (Col2, Row2)), Ok(()));
    }
}
