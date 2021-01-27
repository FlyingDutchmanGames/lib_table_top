use enum_map::EnumMap;
use thiserror::Error;

#[derive(Copy, Clone, Debug, Enum, PartialEq, Eq)]
pub enum Marker {
    X,
    O,
}

impl Marker {
    pub fn opponent(&self) -> Self {
        match self {
            X => O,
            O => X,
        }
    }
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
    pub const ALL: [Self; 3] = [Row0, Row1, Row2];
}

use Row::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum)]
pub enum Col {
    Col0,
    Col1,
    Col2,
}

impl Col {
    pub const ALL: [Self; 3] = [Col0, Col1, Col2];
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
pub type Board = EnumMap<Col, EnumMap<Row, Option<Marker>>>;

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameState {
    pub history: Vec<(Marker, Position)>,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            history: Vec::with_capacity(9),
        }
    }

    pub fn board(&self) -> Board {
        let mut board = enum_map! { _ => enum_map! { _ => None }};

        for &(marker, (col, row)) in &self.history {
            board[col][row] = Some(marker);
        }

        board
    }

    pub fn available(&self) -> Vec<Position> {
        iproduct!(&Col::ALL, &Row::ALL)
            .map(|(&col, &row)| (col, row))
            .filter(|position| !self.is_position_taken(position))
            .collect()
    }

    pub fn whose_turn(&self) -> Option<Marker> {
        if self.is_full() {
            return None;
        }

        self.history
            .last()
            .map(|(marker, _pos)| marker.opponent())
            .or(Some(X))
    }

    pub fn status(&self) -> Status {
        let board = self.board();

        let win = POSSIBLE_WINS
            .iter()
            .filter_map(|&possibility| {
                let [a, b, c] = possibility.map(|(col, row)| board[col][row]);
                if a == b && b == c {
                    a.map(|marker| Status::Win {
                        marker,
                        spaces: possibility,
                    })
                } else {
                    None
                }
            })
            .nth(0);

        win.unwrap_or_else(|| if self.is_full() { Draw } else { InProgress })
    }

    fn is_full(&self) -> bool {
        self.history.len() == 9
    }

    fn is_position_taken(&self, position: &Position) -> bool {
        self.history.iter().any(|(_marker, pos)| pos == position)
    }
}

impl GameState {
    pub fn undo(&mut self) -> Option<(Marker, Position)> {
        self.history.pop()
    }

    pub fn make_move(&mut self, marker: Marker, position: Position) -> Result<(), Error> {
        if self.is_position_taken(&position) {
            return Err(SpaceIsTaken);
        }
        match self.whose_turn() {
            Some(current_turn) if current_turn == marker => {
                self.history.push((marker, position));
                Ok(())
            }
            _ => Err(OtherPlayerTurn { attempted: marker }),
        }
    }
}
