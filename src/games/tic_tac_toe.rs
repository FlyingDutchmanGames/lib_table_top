use enum_map::EnumMap;
use im::Vector;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use thiserror::Error;

/// Player pieces, X & O
#[derive(Copy, Clone, Debug, Enum, PartialEq, Eq, Serialize, Deserialize)]
pub enum Player {
    X,
    O,
}

impl Player {
    /// Returns the opposite player
    /// ```
    /// use lib_table_top::games::tic_tac_toe::Player::*;
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

use Player::*;

/// Various Errors that can happen from invalid actions being applied to the game
#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    /// Returned when trying to claim an already claimed space
    #[error("space ({:?}, {:?}) is taken", attempted.0, attempted.1)]
    SpaceIsTaken { attempted: Position },
    /// Returned when the wrong player tries to take a turn
    #[error("not {:?}'s turn", attempted)]
    OtherPlayerTurn { attempted: Player },
}

use Error::*;

/// A `Row` of the Tic-Tac-Toe board
#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Row {
    Row0 = 0,
    Row1 = 1,
    Row2 = 2,
}

/// All the rows of the board
impl Row {
    pub const ALL: [Self; 3] = [Row0, Row1, Row2];
}

use Row::*;

/// A `Col` of the Tic-Tac-Toe board
#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Col {
    Col0 = 0,
    Col1 = 1,
    Col2 = 2,
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
pub type Board = EnumMap<Col, EnumMap<Row, Option<Player>>>;
/// An action being taken by a player to claim a position
pub type Action = (Player, Position);

/// The three states a game can be in
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// There are still available positions to be claimed on the board
    InProgress,
    /// All positions have been claimed and there is no winner
    Draw,
    /// All positions have been claimed and there *is* a winner
    Win {
        player: Player,
        positions: [Position; 3],
    },
}

use Status::*;

/// Representation of a Tic-Tac-Toe game
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameState {
    history: Vector<Position>,
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
            history: Vector::new(),
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
    /// let game = game.make_move(action1).unwrap();
    /// let action2 = game.valid_actions().next().unwrap();
    /// let game = game.make_move(action2).unwrap();
    /// let action3 = game.valid_actions().next().unwrap();
    /// let game = game.make_move(action3).unwrap();
    ///
    /// assert_eq!(game.history().count(), 3);
    /// assert_eq!(
    ///   game.history().collect::<Vec<Action>>(),
    ///   vec![action1, action2, action3]
    /// )
    /// ```
    pub fn history(&self) -> impl Iterator<Item = Action> + '_ {
        let players = [X, O].iter().cycle();
        self.history
            .iter()
            .zip(players)
            .map(|(&position, &player)| (player, position))
    }

    /// Maps Col => Row => Players for the current state of the game
    /// ```
    /// use lib_table_top::games::tic_tac_toe::{GameState, Row, Row::*, Col, Col::*, Player::*};
    ///
    /// let mut game: GameState = Default::default();
    ///
    /// // All spaces are empty on a new game
    /// let board = game.board();
    ///
    /// for &col in &Col::ALL {
    ///   for &row in &Row::ALL {
    ///     assert_eq!(board[col][row], None);
    ///   }
    /// }
    ///
    /// // After making moves they're returned in the board
    /// assert_eq!(game.board()[Col1][Row1], None);
    /// let game = game.make_move((X, (Col1, Row1))).unwrap();
    /// assert_eq!(game.board()[Col1][Row1], Some(X));
    /// ```
    pub fn board(&self) -> Board {
        let mut board = enum_map! { _ => enum_map! { _ => None }};

        self.history().for_each(|(player, (col, row))| {
            board[col][row] = Some(player);
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
    /// let game = game.make_move(action).unwrap();
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
    ///   GameState, Action, Row::*, Col::*, Player::*
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
    /// use lib_table_top::games::tic_tac_toe::{GameState, Player::*};
    ///
    /// // Games always start with `X`
    /// let mut game: GameState = Default::default();
    /// assert_eq!(game.whose_turn(), X);
    ///
    /// // After X moves, it's O's turn
    /// let action = game.valid_actions().next().unwrap();
    /// let game = game.make_move(action).unwrap();
    ///
    /// assert_eq!(game.whose_turn(), O);
    /// ```
    pub fn whose_turn(&self) -> Player {
        if self.history.len() % 2 == 0 {
            X
        } else {
            O
        }
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
                    a.map(|player| Win { player, positions })
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
        self.history.iter().any(|pos| pos == position)
    }
}

impl GameState {
    /// Undo the previous action and yield the action. Returns `None` if there is no previous
    /// action
    /// ```
    /// use lib_table_top::games::tic_tac_toe::GameState;
    ///
    /// // A new game has no history, so there is nothing to do
    /// let mut game: GameState = Default::default();
    /// assert_eq!(game.undo(), None);
    ///
    /// // You can undo actions
    /// let original_game = game.clone();
    /// assert!(game == original_game);
    ///
    /// let action = game.valid_actions().next().unwrap();
    /// let mut game = game.make_move(action).unwrap();
    /// assert!(game != original_game);
    ///
    /// assert_eq!(game.undo(), Some(action));
    /// assert_eq!(game, original_game);
    /// ```
    pub fn undo(&mut self) -> Option<Action> {
        // NGL, this one is tricky, because once you pop(), it switches whose turn it is.
        let current_player = self.whose_turn().opponent();
        self.history.pop_back().map(|pos| (current_player, pos))
    }

    /// Apply an action to the game, returns nothing if successful, and returns an error and
    /// doesn't change the game state if there is an issue with the action
    /// ```
    /// use lib_table_top::games::tic_tac_toe::{
    ///   GameState, Error::*, Player::*, Row::*, Col::*
    /// };
    ///
    /// let game: GameState = Default::default();
    ///
    /// // If the wrong player tries to make a move
    /// let result = game.make_move((game.whose_turn().opponent(), (Col0, Row0)));
    /// assert_eq!(result, Err(OtherPlayerTurn { attempted: O }));
    /// assert_eq!(&result.unwrap_err().to_string(), "not O's turn");
    ///
    /// // The correct player can make a move
    /// let pos = (Col0, Row0);
    /// let result = game.make_move((game.whose_turn(), pos));
    /// assert!(result.is_ok());
    /// let game = result.unwrap();
    ///
    /// // Trying to make a move on a taken space yields an error
    /// assert!(!game.available().any(|x| x == pos));
    /// let result = game.make_move((game.whose_turn(), pos));
    /// assert_eq!(result, Err(SpaceIsTaken { attempted: pos }));
    /// assert_eq!(&result.unwrap_err().to_string(), "space (Col0, Row0) is taken");
    /// ```
    pub fn make_move(&self, (player, position): Action) -> Result<Self, Error> {
        if self.is_position_taken(&position) {
            return Err(SpaceIsTaken {
                attempted: position,
            });
        }

        if player == self.whose_turn() {
            let mut new_game_state = self.clone();
            new_game_state.history.push_back(position);
            Ok(new_game_state)
        } else {
            Err(OtherPlayerTurn { attempted: player })
        }
    }
}
