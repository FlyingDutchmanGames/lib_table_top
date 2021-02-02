use enum_map::EnumMap;
use thiserror::Error;

/// Player pieces, X & O
#[derive(Copy, Clone, Debug, Enum, PartialEq, Eq)]
pub enum Marker {
    X,
    O,
}

impl Marker {
    /// Returns the opposite player
    /// ```
    /// use lib_table_top::games::tic_tac_toe::Marker::*;
    ///
    /// assert_eq!(X, O.opponent());
    /// assert_eq!(O, X.opponent());
    /// ```
    pub fn opponent(&self) -> Self {
        match self {
            X => O,
            O => X,
        }
    }
}

use Marker::*;

/// Various Errors that can happen from invalid actions being applied to the game
#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    /// Returned when trying to claim an already claimed space
    #[error("Space is taken")]
    SpaceIsTaken,
    /// Returned when the wrong player tries to take a turn
    #[error("Not {:?}'s turn", attempted)]
    OtherPlayerTurn { attempted: Marker },
}

use Error::*;

/// A `Row` of the Tic-Tac-Toe board
#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum)]
pub enum Row {
    Row0,
    Row1,
    Row2,
}

/// All the rows of the board
impl Row {
    pub const ALL: [Self; 3] = [Row0, Row1, Row2];
}

use Row::*;

/// A `Col` of the Tic-Tac-Toe board
#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum)]
pub enum Col {
    Col0,
    Col1,
    Col2,
}

/// All the cols of the board
impl Col {
    pub const ALL: [Self; 3] = [Col0, Col1, Col2];
}

use Col::*;

/// All 8 possible ways to win Tic-Tac-Toe
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

/// A type representing a position on the board, denoted in terms of (x, y)
pub type Position = (Col, Row);
/// A representation of the Tic-Tac-Toe Board
pub type Board = EnumMap<Col, EnumMap<Row, Option<Marker>>>;
/// An action being taken by a marker to claim a position
pub type Action = (Marker, Position);

/// The three states a game can be in
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// There are still available positions to be claimed on the board
    InProgress,
    /// All positions have been claimed and there is no winner
    Draw,
    /// All positions have been claimed and there *is* a winner
    Win {
        marker: Marker,
        positions: [Position; 3],
    },
}

use Status::*;

/// Representation of a Tic-Tac-Toe game
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    history: Vec<Action>,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    /// Make a new Tic-Tac-Toe game, this is the same as the Default::default implementation
    /// ```
    /// use lib_table_top::games::tic_tac_toe::GameState;
    ///
    /// let game1 = GameState::new();
    /// let game2: GameState = Default::default();
    /// assert_eq!(game1, game2);
    /// ```
    pub fn new() -> Self {
        GameState {
            history: Vec::with_capacity(9),
        }
    }

    /// An iterator over the actions that have been taken on the game, starting from the beginning
    /// of the game
    /// ```
    /// use lib_table_top::games::tic_tac_toe::{Action, GameState};
    ///
    /// let mut game: GameState = Default::default();
    ///
    /// // The history starts empty
    /// assert!(game.history().count() == 0);
    ///
    /// // THe history can be iterated in order
    /// let action1 = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action1).is_ok());
    /// let action2 = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action2).is_ok());
    /// let action3 = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action3).is_ok());
    ///
    /// assert_eq!(game.history().count(), 3);
    /// assert_eq!(
    ///   game.history().collect::<Vec<&Action>>(),
    ///   vec![&action1, &action2, &action3]
    /// )
    /// ```
    pub fn history(&self) -> impl Iterator<Item = &Action> + Clone {
        self.history.iter()
    }

    pub fn board(&self) -> Board {
        let mut board = enum_map! { _ => enum_map! { _ => None }};

        self.history.iter().for_each(|&(marker, (col, row))| {
            board[col][row] = Some(marker);
        });

        board
    }

    /// An iterator over the available positions on the board
    /// ```
    /// use lib_table_top::games::tic_tac_toe::GameState;
    ///
    /// let mut game: GameState = Default::default();
    /// let board = game.board();
    ///
    /// for (col, row) in game.available() {
    ///   assert_eq!(board[col][row], None);
    /// }
    ///
    /// // Spaces get taken as the game goes on
    /// assert_eq!(game.available().count(), 9);
    ///
    /// let action = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action).is_ok());
    ///
    /// assert_eq!(game.available().count(), 8);
    /// ```
    pub fn available(&self) -> impl Iterator<Item = Position> + Clone + '_ {
        iproduct!(&Col::ALL, &Row::ALL)
            .map(|(&col, &row)| (col, row))
            .filter(move |&position| !self.is_position_taken(&position))
    }

    /// An iterator over the valid actions that can be played during the next turn
    /// ```
    /// use lib_table_top::games::tic_tac_toe::{
    ///   GameState, Action, Row::*, Col::*, Marker::*
    /// };
    ///
    /// let game: GameState = Default::default();
    /// assert_eq!(
    ///   game.valid_actions().collect::<Vec<Action>>(),
    ///   vec![
    ///     (X, (Col0, Row0)),
    ///     (X, (Col0, Row1)),
    ///     (X, (Col0, Row2)),
    ///     (X, (Col1, Row0)),
    ///     (X, (Col1, Row1)),
    ///     (X, (Col1, Row2)),
    ///     (X, (Col2, Row0)),
    ///     (X, (Col2, Row1)),
    ///     (X, (Col2, Row2))
    ///   ]
    /// );
    /// ```
    pub fn valid_actions(&self) -> impl Iterator<Item = Action> + Clone + '_ {
        let whose_turn = self.whose_turn();
        self.available().map(move |action| (whose_turn, action))
    }

    /// Returns the player who plays the next turn, games always start with `X`
    /// ```
    /// use lib_table_top::games::tic_tac_toe::{GameState, Marker::*};
    ///
    /// // Games always start with `X`
    /// let mut game: GameState = Default::default();
    /// assert_eq!(game.whose_turn(), X);
    ///
    /// // After X moves, it's O's turn
    /// let action = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action).is_ok());
    ///
    /// assert_eq!(game.whose_turn(), O);
    /// ```
    pub fn whose_turn(&self) -> Marker {
        self.history
            .last()
            .map(|(marker, _pos)| marker.opponent())
            .unwrap_or(X)
    }

    /// Returns the status of the current game, see [`Status`](enum@Status) for more details
    /// ```
    /// use lib_table_top::games::tic_tac_toe::{GameState, Status};
    ///
    /// let game: GameState = Default::default();
    /// assert_eq!(game.status(), Status::InProgress);
    /// ```
    pub fn status(&self) -> Status {
        let board = self.board();

        POSSIBLE_WINS
            .iter()
            .filter_map(|&positions| {
                let [a, b, c] = positions.map(|(col, row)| board[col][row]);

                if a == b && b == c {
                    a.map(|marker| Win { marker, positions })
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_else(|| if self.is_full() { Draw } else { InProgress })
    }

    fn is_full(&self) -> bool {
        self.history.len() == 9
    }

    fn is_position_taken(&self, position: &Position) -> bool {
        self.history.iter().any(|(_marker, pos)| pos == position)
    }
}

impl GameState {
    pub fn undo(&mut self) -> Option<Action> {
        self.history.pop()
    }

    pub fn make_move(&mut self, (marker, position): Action) -> Result<(), Error> {
        if self.is_position_taken(&position) {
            return Err(SpaceIsTaken);
        }

        if marker == self.whose_turn() {
            self.history.push((marker, position));
            Ok(())
        } else {
            Err(OtherPlayerTurn { attempted: marker })
        }
    }
}
