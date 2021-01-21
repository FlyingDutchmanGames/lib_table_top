use colored::*;
use enum_map::EnumMap;
use std::fmt;
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
    #[error("not {:?}'s turn", attempted)]
    OtherPlayerTurn { attempted: Marker },
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

pub const POSSIBLE_WINS: [[(Col, Row); 3]; 8] = [
    // Fill up a row
    [(Col0, Row0), (Col0, Row1), (Col0, Row2)],
    [(Col1, Row0), (Col1, Row1), (Col1, Row2)],
    [(Col2, Row0), (Col2, Row1), (Col2, Row2)],
    // Fill up a col
    [(Col0, Row0), (Col1, Row0), (Col2, Row0)],
    [(Col0, Row1), (Col1, Row1), (Col2, Row1)],
    [(Col0, Row2), (Col1, Row2), (Col2, Row2)],
    // Diagonal
    [(Col0, Row0), (Col1, Row1), (Col2, Row2)],
    [(Col2, Row0), (Col1, Row1), (Col0, Row2)],
];

pub type Position = (Col, Row);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    InProgress,
    Draw,
    Win {
        marker: Marker,
        spaces: [(Col, Row); 3],
    },
}

use Status::*;

pub struct GameState {
    board: EnumMap<Col, EnumMap<Row, Option<Marker>>>,
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let status = self.status();

        let markers: Vec<Vec<ColoredString>> = self
            .board
            .iter()
            .map(|(col_num, row)| {
                row.iter()
                    .map(|(row_num, marker)| {
                        let mark = match marker {
                            Some(X) => "X".blue(),
                            Some(O) => "O".red(),
                            None => "*".normal(),
                        };

                        if let Win { spaces, .. } = status {
                            if spaces.contains(&(col_num, row_num)) {
                                return mark.bold();
                            }
                        }

                        mark
                    })
                    .collect()
            })
            .collect();
        write!(
            f,
            r#"
2  {} {} {}
1  {} {} {}
0  {} {} {}
   0 1 2
"#,
            markers[0][2],
            markers[1][2],
            markers[2][2],
            markers[0][1],
            markers[1][1],
            markers[2][1],
            markers[0][0],
            markers[1][0],
            markers[2][0]
        )
    }
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            board: enum_map! { _ => enum_map! { _ => None } },
        }
    }

    pub fn board(&self) -> [[Option<Marker>; 3]; 3] {
        [
            [
                self.board[Col0][Row0],
                self.board[Col0][Row1],
                self.board[Col0][Row2],
            ],
            [
                self.board[Col1][Row0],
                self.board[Col1][Row1],
                self.board[Col1][Row2],
            ],
            [
                self.board[Col2][Row0],
                self.board[Col2][Row1],
                self.board[Col2][Row2],
            ],
        ]
    }

    pub fn at_position(&self, (col, row): Position) -> Option<Marker> {
        self.board[col][row]
    }

    pub fn available(&self) -> Vec<Position> {
        iproduct!(&Col::ALL, &Row::ALL)
            .filter(|&(&col, &row)| self.board[col][row].is_none())
            .map(|(&col, &row)| (col, row))
            .collect()
    }

    pub fn whose_turn(&self) -> Option<Marker> {
        if self.is_full() {
            return None;
        }

        let mut count: EnumMap<Marker, u8> = enum_map! { _ => 0 };

        self.board
            .iter()
            .flat_map(|(_col_num, row)| row.iter())
            .filter_map(|(_row_num, &marker)| marker)
            .for_each(|marker| count[marker] += 1);

        if count[X] == count[O] {
            Some(X)
        } else {
            Some(O)
        }
    }

    pub fn status(&self) -> Status {
        let win = POSSIBLE_WINS
            .iter()
            .filter_map(|&possibility| {
                let [a, b, c] = possibility.map(|position| self.at_position(position));
                if a == b && b == c {
                    a.map(|marker| Win {
                        marker,
                        spaces: possibility,
                    })
                } else {
                    None
                }
            })
            .nth(0);

        if let Some(win) = win {
            return win;
        } else {
        }

        match win {
            Some(win) => win,
            None => {
                if self.is_full() {
                    Draw
                } else {
                    InProgress
                }
            }
        }
    }

    pub fn make_move(&mut self, marker: Marker, (col, row): Position) -> Result<(), Error> {
        if self.at_position((col, row)).is_some() {
            return Err(SpaceIsTaken);
        }
        match self.whose_turn() {
            Some(current_turn) if current_turn == marker => {
                self.board[col][row] = Some(marker);
                Ok(())
            }
            _ => Err(OtherPlayerTurn { attempted: marker }),
        }
    }

    fn is_full(&self) -> bool {
        iproduct!(&Col::ALL, &Row::ALL).all(|(&col, &row)| self.at_position((col, row)).is_some())
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
        assert_eq!(game_state.whose_turn(), Some(X));
        assert_eq!(game_state.make_move(X, (Col1, Row1)), Ok(()));

        assert_eq!(game_state.whose_turn(), Some(O));

        assert_eq!(game_state.make_move(O, (Col1, Row1)), Err(SpaceIsTaken));
        assert_eq!(
            game_state.make_move(X, (Col1, Row2)),
            Err(OtherPlayerTurn { attempted: X })
        );

        assert_eq!(game_state.make_move(O, (Col2, Row2)), Ok(()));
    }

    #[test]
    fn test_you_can_format_it() {
        // new game
        let mut game_state = GameState::new();
        println!("{}", game_state);
        assert_eq!(
            format!("{}", game_state),
            "\n2  * * *\n1  * * *\n0  * * *\n   0 1 2\n"
        );

        // a couple of moves
        let _ = game_state.make_move(X, (Col0, Row0));
        let _ = game_state.make_move(O, (Col2, Row2));
        println!("{}", game_state);
        assert_eq!(
            format!("{}", game_state),
            "\n2  * * \u{1b}[31mO\u{1b}[0m\n1  * * *\n0  \u{1b}[34mX\u{1b}[0m * *\n   0 1 2\n"
        );

        // with a win
        let _ = game_state.make_move(X, (Col0, Row1));
        let _ = game_state.make_move(O, (Col2, Row1));
        let _ = game_state.make_move(X, (Col0, Row2));

        println!("{}", game_state);
        assert_eq!(
            format!("{}", game_state),
            "\n2  \u{1b}[1;34mX\u{1b}[0m * \u{1b}[31mO\u{1b}[0m\n1  \u{1b}[1;34mX\u{1b}[0m * \u{1b}[31mO\u{1b}[0m\n0  \u{1b}[1;34mX\u{1b}[0m * *\n   0 1 2\n"
        );
    }
}
