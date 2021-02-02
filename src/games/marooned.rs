use enum_map::EnumMap;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use thiserror::Error;

/// A row value inside of a position (y coordinate)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Row(pub u8);

/// A col value inside of a position (x coordinate)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Col(pub u8);

/// A position on the board denoted in column, then row (x, y)
pub type Position = (Col, Row);

/// Players 1 and 2
#[derive(
    Copy, Clone, Debug, Enum, PartialEq, Eq, PartialOrd, Ord, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum Player {
    /// Player One
    P1 = 1,
    /// Player Two
    P2 = 2,
}

use Player::*;

impl Player {
    /// Return the opponent (opposite) player
    /// ```
    /// use lib_table_top::games::marooned::Player::*;
    ///
    /// assert_eq!(P1.opponent(), P2);
    /// assert_eq!(P2.opponent(), P1);
    /// ```
    pub fn opponent(&self) -> Self {
        match self {
            P1 => P2,
            P2 => P1,
        }
    }
}

/// The various errors that can be returned from invalid Marooned settings
#[derive(Error, Debug, PartialEq, Eq)]
pub enum SettingsError {
    /// When dimensions are invalid, rows * cols must be >= 2
    #[error("Invalid dimensions")]
    InvalidDimensions,
    /// You can't remove a position that isn't on the board
    #[error("Cant remove the position ({:?}) because it isn't on the board", pos)]
    CantRemovePositionNotOnBoard { pos: Position },
    /// Two players can't start on the same position
    #[error("Players must start at different positions")]
    PlayersCantStartAtSamePosition,
    /// A player can't start off the board
    #[error("Players must start on board, but {:?} is on {:?}", player, position)]
    PlayersMustStartOnBoard { player: Player, position: Position },
    /// A player can't start on a removed square
    #[error("Can't start player {:?} on removed position {:?}", player, position)]
    PlayerCantStartOnRemovedSquare { player: Player, position: Position },
}

use SettingsError::*;

/// Representation of the dimensions of the game board
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimensions {
    pub rows: u8,
    pub cols: u8,
}

impl Dimensions {
    /// Create new Dimensions
    ///
    /// No dimension may be equal to 0, and rows * cols must be >= 2
    /// ```
    /// use lib_table_top::games::marooned::{Dimensions, SettingsError};
    ///
    /// assert_eq!(Dimensions::new(3, 4), Ok(Dimensions { rows: 3, cols: 4 }));
    /// assert_eq!(Dimensions::new(0, 0), Err(SettingsError::InvalidDimensions));
    /// assert_eq!(Dimensions::new(0, 9), Err(SettingsError::InvalidDimensions));
    /// assert_eq!(Dimensions::new(9, 0), Err(SettingsError::InvalidDimensions));
    /// assert_eq!(Dimensions::new(1, 1), Err(SettingsError::InvalidDimensions));
    /// ```
    pub fn new(rows: u8, cols: u8) -> Result<Self, SettingsError> {
        match (rows, cols) {
            (0, _) => Err(InvalidDimensions),
            (_, 0) => Err(InvalidDimensions),
            (1, 1) => Err(InvalidDimensions),
            _ => Ok(Self { rows, cols }),
        }
    }

    /// An iterator over all of the positions that are on the board, includes
    /// removed/currently occupied positions
    /// ```
    ///
    /// use lib_table_top::games::marooned::{Dimensions, Position, Row, Col};
    ///
    /// let dimensions = Dimensions { rows: 2, cols: 2 };
    /// assert_eq!(
    ///   dimensions.all_positions().collect::<Vec<Position>>(),
    ///   vec![(Col(0), Row(0)), (Col(0), Row(1)), (Col(1), Row(0)), (Col(1), Row(1))]
    /// )
    /// ```
    pub fn all_positions(&self) -> impl Iterator<Item = Position> + Clone {
        iproduct!(0..self.cols, 0..self.rows).map(|(col, row)| (Col(col), Row(row)))
    }

    /// Returns whether a position is on the board
    /// ```
    /// use lib_table_top::games::marooned::{Dimensions, Col, Row};
    ///
    /// let dimensions = Dimensions { rows: 2, cols: 2 };
    /// assert!(dimensions.is_position_on_board((Col(0), Row(0))));
    /// assert!(dimensions.is_position_on_board((Col(1), Row(1))));
    /// assert!(!dimensions.is_position_on_board((Col(2), Row(2))));
    /// ```
    pub fn is_position_on_board(&self, position: Position) -> bool {
        let (Col(col), Row(row)) = position;
        row < self.rows && col < self.cols
    }

    /// An iterator over the positions contained within the board that are adjacent to the given
    /// position, does not include the given position
    /// ```
    /// use lib_table_top::games::marooned::{Dimensions, Row, Col, Position};
    ///
    /// let dimensions = Dimensions { rows: 3, cols: 3 };
    ///
    /// assert_eq!(
    ///     dimensions
    ///         .adjacenct_positions((Col(0), Row(0)))
    ///         .collect::<Vec<Position>>(),
    ///     vec![(Col(1), Row(1)), (Col(1), Row(0)), (Col(0), Row(1))]
    /// );
    ///
    /// assert_eq!(
    ///     dimensions
    ///         .adjacenct_positions((Col(1), Row(1)))
    ///         .collect::<Vec<Position>>(),
    ///     vec![
    ///         (Col(2), Row(2)),
    ///         (Col(2), Row(1)),
    ///         (Col(2), Row(0)),
    ///         (Col(1), Row(2)),
    ///         (Col(1), Row(0)),
    ///         (Col(0), Row(2)),
    ///         (Col(0), Row(1)),
    ///         (Col(0), Row(0))
    ///     ]
    /// );
    ///
    /// ```
    pub fn adjacenct_positions(
        &self,
        (Col(col), Row(row)): Position,
    ) -> impl Iterator<Item = Position> + Clone + '_ {
        iproduct!(
            Self::checked_adjacent(col, self.cols),
            Self::checked_adjacent(row, self.rows)
        )
        .filter(move |&(c, r)| (c, r) != (col, row))
        .map(|(c, r)| (Col(c), Row(r)))
    }

    fn default_player_starting_positions(&self) -> EnumMap<Player, Position> {
        let col_midpoint = ((self.cols - 1) as f64) / 2f64;

        enum_map! {
            P1 => (Col(col_midpoint.ceil() as u8), Row(0)),
            P2 => (Col(col_midpoint.floor() as u8), Row(self.rows - 1)),
        }
    }

    fn checked_adjacent(starting_offset: u8, max: u8) -> impl Iterator<Item = u8> + Clone {
        let vals = vec![
            starting_offset.checked_add(1),
            Some(starting_offset),
            starting_offset.checked_sub(1),
        ];

        vals.into_iter()
            .filter_map(|offset| offset)
            .filter(move |&offset| offset < max)
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self { rows: 8, cols: 6 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub dimensions: Dimensions,
    starting_player_positions: EnumMap<Player, Position>,
    starting_removed_positions: Vec<Position>,
}

/// Tools to build Marooned games
///
/// ```
/// use lib_table_top::games::marooned::{Dimensions, SettingsBuilder, Col, Row};
///
/// assert!(SettingsBuilder::new()
///    .rows(3)
///    .cols(4)
///    .p1_starting((Col(0), Row(0)))
///    .p2_starting((Col(1), Row(1)))
///    .starting_removed_positions(vec![(Col(2), Row(2))])
///    .build_game()
///    .is_ok()
/// )
/// ```
///
/// If you're trying to play with the default settings, it's easiest to use the Default
/// implemenation provided by [`GameState`](struct@GameState)
/// ```
/// use lib_table_top::games::marooned::GameState;
///
/// let game: GameState = Default::default();
/// ```
#[derive(Clone, Debug)]
pub struct SettingsBuilder {
    rows: u8,
    cols: u8,
    p1_starting: Option<Position>,
    p2_starting: Option<Position>,
    starting_removed_positions: Vec<Position>,
}

impl Default for SettingsBuilder {
    fn default() -> Self {
        let Dimensions { rows, cols } = Default::default();
        Self {
            cols,
            rows,
            p1_starting: None,
            p2_starting: None,
            starting_removed_positions: Default::default(),
        }
    }
}

impl SettingsBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn rows(mut self, rows: u8) -> Self {
        self.rows = rows;
        self
    }
    pub fn cols(mut self, cols: u8) -> Self {
        self.cols = cols;
        self
    }

    pub fn starting_removed_positions(mut self, positions: Vec<Position>) -> Self {
        self.starting_removed_positions = positions;
        self
    }

    pub fn p1_starting(mut self, pos: Position) -> Self {
        self.p1_starting = Some(pos);
        self
    }

    pub fn p2_starting(mut self, pos: Position) -> Self {
        self.p2_starting = Some(pos);
        self
    }

    pub fn build(self) -> Result<Settings, SettingsError> {
        Settings::new(self)
    }

    pub fn build_game(self) -> Result<GameState, SettingsError> {
        self.build().map(|settings| GameState::new(settings))
    }
}

impl Settings {
    pub fn new(builder: SettingsBuilder) -> Result<Self, SettingsError> {
        let dimensions = Dimensions::new(builder.rows, builder.cols)?;
        let default_starting = dimensions.default_player_starting_positions();
        let p1_starting = builder.p1_starting.unwrap_or(default_starting[P1]);
        let p2_starting = builder.p2_starting.unwrap_or(default_starting[P2]);
        let starting_player_positions = enum_map! { P1 => p1_starting, P2 => p2_starting };

        for &pos in &builder.starting_removed_positions {
            if !dimensions.is_position_on_board(pos) {
                return Err(CantRemovePositionNotOnBoard { pos });
            }
        }
        for (player, position) in starting_player_positions {
            if !dimensions.is_position_on_board(position) {
                return Err(PlayersMustStartOnBoard { player, position });
            }

            if builder.starting_removed_positions.contains(&position) {
                return Err(PlayerCantStartOnRemovedSquare { player, position });
            }
        }

        let mut starting_removed_positions = builder.starting_removed_positions;
        starting_removed_positions.sort();
        starting_removed_positions.dedup();

        if starting_player_positions[P1] == starting_player_positions[P2] {
            return Err(PlayersCantStartAtSamePosition);
        }

        Ok(Self {
            dimensions,
            starting_player_positions,
            starting_removed_positions,
        })
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dimensions: Default::default(),
            starting_player_positions: enum_map! {
                P1 => (Col(2), Row(0)),
                P2 => (Col(3), Row(7)),
            },
            starting_removed_positions: Default::default(),
        }
    }
}

/// Action that player makes on the game
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub player: Player,
    pub to: Position,
    pub remove: Position,
}

/// The current status of the game
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// The game is still in progress
    InProgress,
    /// The game is over, no more actions can be taken on this game
    Win { player: Player },
}

use Status::*;

/// The game state
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GameState {
    pub settings: Settings,
    history: Vec<Action>,
}

impl GameState {
    /// Makes a new game, you're better off using [`SettingsBuilder`](struct@SettingsBuilder) to
    /// construct a new game
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            history: vec![],
        }
    }

    /// Returns the current status of a game
    /// ```
    /// use lib_table_top::games::marooned::{GameState, Status, SettingsBuilder, Player::*};
    ///
    /// // A new default game is in progress
    /// let game: GameState = Default::default();
    /// assert_eq!(game.status(), Status::InProgress);
    ///
    /// // A game with no more available spaces to move for the current player is over
    /// let game = SettingsBuilder::new().rows(1).cols(2).build_game().unwrap();
    /// assert_eq!(game.status(), Status::Win { player: P2 })
    /// ```
    pub fn status(&self) -> Status {
        let current_player = self.whose_turn();

        if self
            .allowed_movement_targets_for_player(current_player)
            .next()
            == None
        {
            Win {
                player: current_player.opponent(),
            }
        } else {
            InProgress
        }
    }

    /// Returns the player who's turn it currently is. All games start with P1
    /// ```
    /// use lib_table_top::games::marooned::{Player, GameState};
    ///
    /// let game: GameState = Default::default();
    /// assert_eq!(game.whose_turn(), Player::P1);
    /// ```
    pub fn whose_turn(&self) -> Player {
        self.history
            .last()
            .map(|Action { player, .. }| player.opponent())
            .unwrap_or(P1)
    }

    /// An iterator over the actions made, in order, starting from the beginning of the game
    /// ```
    /// use lib_table_top::games::marooned::{GameState, Action};
    ///
    /// let mut game: GameState = Default::default();
    ///
    /// // History starts empty
    /// assert_eq!(game.history().count(), 0);
    ///
    /// // Apply some actions
    /// let action_1 = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action_1).is_ok());
    ///
    /// let action_2 = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action_2).is_ok());
    ///
    /// let action_3 = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action_3).is_ok());
    ///
    /// // `game.history()` is an iterator over the actions in order
    /// assert_eq!(
    ///   game.history().collect::<Vec<&Action>>(),
    ///   vec![&action_1, &action_2, &action_3]
    /// )
    /// ```
    pub fn history(&self) -> impl Iterator<Item = &Action> + Clone {
        self.history.iter()
    }

    /// Returns an iterator of the positions that have already been removed
    /// ```
    /// use lib_table_top::games::marooned::{GameState, Position, SettingsBuilder, Row, Col};
    ///
    /// // The default game settings start with no removed positions
    /// let game: GameState = Default::default();
    /// let removed: Vec<Position> = game.removed_positions().collect();
    /// assert_eq!(removed, vec![]);
    ///
    /// // You can start with some already removed
    /// let pos = (Col(1), Row(1));
    /// let game = SettingsBuilder::new().starting_removed_positions(vec![pos]).build_game().unwrap();
    /// let removed: Vec<Position> = game.removed_positions().collect();
    /// assert_eq!(removed, vec![pos]);
    /// ```
    pub fn removed_positions(&self) -> impl Iterator<Item = Position> + Clone + '_ {
        self.settings
            .starting_removed_positions
            .iter()
            .chain(self.history.iter().map(|Action { remove, .. }| remove))
            .copied()
    }

    /// Calls `removable_positions_for_player` with the current player
    pub fn removable_positions(&self) -> impl Iterator<Item = Position> + Clone + '_ {
        self.removable_positions_for_player(self.whose_turn())
    }

    /// Returns an iterator of removable positions for a player. Players can not remove the space
    /// their opponent is on, but can remove they space they are currently on
    /// ```
    /// use lib_table_top::games::marooned::{SettingsBuilder, Row, Col, Player::*, Position};
    ///
    /// let game = SettingsBuilder::new().rows(2).cols(2).build_game().unwrap();
    /// let removable: Vec<Position> = game.removable_positions_for_player(P1).collect();
    /// assert_eq!(removable, vec![(Col(0), Row(0)), (Col(1), Row(0)), (Col(1), Row(1))]);
    /// ```
    pub fn removable_positions_for_player(
        &self,
        player: Player,
    ) -> impl Iterator<Item = Position> + Clone + '_ {
        self.settings
            .dimensions
            .all_positions()
            .filter(move |&pos| self.is_position_allowed_to_be_removed(pos, player))
    }

    /// Tests whether a position is allowed to be removed by a certain player
    /// ```
    /// use lib_table_top::games::marooned::{GameState, Player::*};
    ///
    /// let game: GameState = Default::default();
    /// for position in game.removable_positions_for_player(P1) {
    ///    assert!(game.is_position_allowed_to_be_removed(position, P1));
    /// }
    /// ```
    pub fn is_position_allowed_to_be_removed(&self, position: Position, player: Player) -> bool {
        (self.settings.dimensions.is_position_on_board(position))
            && (!self.removed_positions().any(|p| p == position))
            && !(self.player_position(player.opponent()) == position)
    }

    /// An iterator over the allowed movements of a player, this takes into account board
    /// dimensions, removed positions, the opponent location
    /// ```
    /// use lib_table_top::games::marooned::{GameState, Position, Row, Col};
    ///
    /// let game: GameState = Default::default();
    /// let movements: Vec<Position> =
    ///     game
    ///     .allowed_movement_targets_for_player(game.whose_turn())
    ///     .collect();
    ///
    /// assert_eq!(movements, vec![
    ///  (Col(3), Row(1)), (Col(3), Row(0)), (Col(2), Row(1)), (Col(1), Row(1)), (Col(1), Row(0))
    /// ]);
    /// ```
    pub fn allowed_movement_targets_for_player(
        &self,
        player: Player,
    ) -> impl Iterator<Item = Position> + Clone + '_ {
        let removed: Vec<Position> = self.removed_positions().collect();
        let other_player_position = self.player_position(player.opponent());

        self.settings
            .dimensions
            .adjacenct_positions(self.player_position(player))
            .filter(move |position| !removed.contains(&position))
            .filter(move |&position| position != other_player_position)
    }

    /// An iterator over all the valid actions the current player can take.
    /// Doesn't return the actions in any particular order, but will return all the actions that
    /// could possibly be valid.
    /// ```
    /// use lib_table_top::games::marooned::{Action, SettingsBuilder, Row, Col, Player::*};
    ///
    /// let game = SettingsBuilder::new().rows(2).cols(2).build_game().unwrap();
    /// let actions: Vec<Action> = game.valid_actions().collect();
    /// assert_eq!(
    ///   actions,
    ///   vec![
    ///     Action { player: P1, to: (Col(1), Row(1)), remove: (Col(0), Row(0)) },
    ///     Action { player: P1, to: (Col(1), Row(1)), remove: (Col(1), Row(0)) },
    ///     Action { player: P1, to: (Col(0), Row(0)), remove: (Col(1), Row(0)) },
    ///     Action { player: P1, to: (Col(0), Row(0)), remove: (Col(1), Row(1)) }
    ///   ]
    /// );
    /// ```
    ///
    /// This can be used to generate a random valid move for an AI
    /// ```
    /// use lib_table_top::games::marooned::{Action, GameState};
    ///
    /// let mut game: GameState = Default::default();
    /// let action: Action = game.valid_actions().next().unwrap();
    /// assert!(game.make_move(action).is_ok());
    /// ```
    pub fn valid_actions(&self) -> impl Iterator<Item = Action> + Clone + '_ {
        let player = self.whose_turn();

        iproduct!(
            self.allowed_movement_targets_for_player(player),
            self.removable_positions()
        )
        .filter(|(to, remove)| to != remove)
        .map(move |(to, remove)| Action { player, to, remove })
    }

    fn player_positions(&self) -> EnumMap<Player, Position> {
        enum_map! {
            P1 => self.player_position(P1),
            P2 => self.player_position(P2),
        }
    }

    /// Returns the position of a player
    /// ```
    /// use lib_table_top::games::marooned::{SettingsBuilder, Row, Col, Player::*};
    ///
    /// let p1_starting = (Col(3), Row(3));
    /// let game = SettingsBuilder::new().p1_starting(p1_starting).build_game().unwrap();
    /// assert_eq!(p1_starting, game.player_position(P1));
    /// ```
    pub fn player_position(&self, player: Player) -> Position {
        self.history
            .iter()
            .rev()
            .filter(|Action { player: p, .. }| p == &player)
            .map(|Action { to, .. }| *to)
            .next()
            .unwrap_or(self.settings.starting_player_positions[player])
    }
}

/// The various things that can go wrong with making a move
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ActionError {
    #[error("Not {:?}'s turn", attempted)]
    OtherPlayerTurn { attempted: Player },
    #[error("{:?} can't move to {:?}", player, target)]
    InvalidMoveToTarget { target: Position, player: Player },
    #[error("Can't remove {:?}", target)]
    InvalidRemove { target: Position },
    #[error("Can't move to the same position as being removed")]
    CantRemoveTheSamePositionAsMoveTo { target: Position },
}

use ActionError::*;

impl GameState {
    /// Moves the game forward by doing an action, returns an error and doesn't do anything if the
    /// action isn't valid for some reason.
    /// ```
    /// use lib_table_top::games::marooned::{Action, GameState, ActionError, Row, Col, Player::*};
    ///
    /// let mut game: GameState = Default::default();
    /// let valid_action = game.valid_actions().next().unwrap();
    ///
    /// // You can't make a move with the wrong player
    /// assert_eq!(
    ///     game.make_move(Action { player: valid_action.player.opponent(), ..valid_action}),
    ///     Err(ActionError::OtherPlayerTurn { attempted: valid_action.player.opponent() })
    /// );
    ///
    /// // You can't move to and remove the same position
    /// assert_eq!(
    ///     game.make_move(Action { to: valid_action.remove, ..valid_action}),
    ///     Err(ActionError::CantRemoveTheSamePositionAsMoveTo { target: valid_action.remove }),
    /// );
    ///
    /// // You can't move to a non adjacent/removed/occupied square
    /// assert_eq!(
    ///     game.make_move(Action { to: (Col(100), Row(100)), ..valid_action}),
    ///     Err(ActionError::InvalidMoveToTarget { target: (Col(100), Row(100)), player: P1})
    /// );
    ///
    /// // You can't remove a position that is already removed/off the board/where the other player
    /// // is standing
    /// assert_eq!(
    ///     game.make_move(Action { remove: (Col(100), Row(100)), ..valid_action}),
    ///     Err(ActionError::InvalidRemove { target: (Col(100), Row(100)) })
    /// );
    ///
    /// // Any valid action advances the game and returns Ok(())
    /// assert!(game.make_move(valid_action).is_ok());
    /// ```
    pub fn make_move(&mut self, action: Action) -> Result<(), ActionError> {
        if action.to == action.remove {
            return Err(CantRemoveTheSamePositionAsMoveTo { target: action.to });
        }

        if action.player != self.whose_turn() {
            return Err(OtherPlayerTurn {
                attempted: action.player,
            });
        }

        if !self
            .allowed_movement_targets_for_player(action.player)
            .any(|pos| action.to == pos)
        {
            return Err(InvalidMoveToTarget {
                player: action.player,
                target: action.to,
            });
        }

        if !self.is_position_allowed_to_be_removed(action.remove, action.player) {
            return Err(InvalidRemove {
                target: action.remove,
            });
        }
        Ok(self.history.push(action))
    }

    /// Allows you to undo the the most recent action, returning the action.
    /// It returns `None` on new games with no actions yet
    /// ```
    /// use lib_table_top::games::marooned::GameState;
    ///
    /// // New games have no actions to undo
    /// let mut game: GameState = Default::default();
    /// assert_eq!(game.undo(), None);
    ///
    /// // You can undo the actions you've made
    /// let next_move = game.valid_actions().next().unwrap();
    /// let original = game.clone();
    /// game.make_move(next_move);
    ///
    /// assert!(original != game);
    /// assert_eq!(game.undo(), Some(next_move));
    /// assert!(original == game);
    /// ```
    pub fn undo(&mut self) -> Option<Action> {
        self.history.pop()
    }
}

impl GameState {
    fn debug_repr(&self) -> String {
        let mut debug_string: String = format!("- Who's Turn: {:?}\n\n", self.whose_turn());

        let rows = 0..self.settings.dimensions.rows;
        let cols = 0..self.settings.dimensions.cols;

        let mut column_labels = String::new();

        column_labels.push_str("   ");
        for col in cols.clone() {
            column_labels.push_str(&format!(" {} ", col));
        }

        debug_string.push_str(&column_labels);
        debug_string.push_str("\n");

        for row in rows.rev() {
            debug_string.push_str(&format!("{} |", row));
            for col in cols.clone() {
                let position = (Col(col), Row(row));
                let marker = if self.player_position(P1) == position {
                    "1"
                } else if self.player_position(P2) == position {
                    "2"
                } else if self.removed_positions().any(|pos| pos == position) {
                    " "
                } else {
                    "*"
                };
                debug_string.push_str(&format!(" {} ", marker));
            }
            debug_string.push_str(&format!("| {}", row));
            debug_string.push_str("\n");
        }

        debug_string.push_str(&column_labels);
        debug_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dimensions() {
        let cases = [
            ((2, 2), [(1, 0), (0, 1)]),
            ((3, 3), [(1, 0), (1, 2)]),
            ((4, 4), [(2, 0), (1, 3)]),
            ((6, 6), [(3, 0), (2, 5)]),
            ((u8::MAX, u8::MAX), [(127, 0), (127, 254)]),
        ];

        for &((rows, cols), [(p1_col, p1_row), (p2_col, p2_row)]) in cases.iter() {
            assert_eq!(
                Dimensions::new(rows, cols)
                    .unwrap()
                    .default_player_starting_positions(),
                enum_map! {
                    P1 => (Col(p1_col), Row(p1_row)),
                    P2 => (Col(p2_col), Row(p2_row))
                }
            )
        }
    }

    #[test]
    fn test_settings_builder_does_validation() {
        assert!(SettingsBuilder::new().build().is_ok());

        for &(rows, cols) in [(0, 10), (10, 0), (0, 0)].iter() {
            assert_eq!(
                SettingsBuilder::new().rows(rows).cols(cols).build(),
                Err(InvalidDimensions)
            );
        }

        for &(rows, cols) in [(2, 1), (1, 2), (2, 2)].iter() {
            assert!(SettingsBuilder::new().rows(rows).cols(cols).build().is_ok());
        }

        assert_eq!(
            SettingsBuilder::new()
                .starting_removed_positions(vec![(Col(100), Row(100))])
                .build(),
            Err(CantRemovePositionNotOnBoard {
                pos: (Col(100), Row(100))
            })
        );

        let pos = (Col(0), Row(0));

        assert_eq!(
            SettingsBuilder::new()
                .p1_starting(pos)
                .starting_removed_positions(vec![pos])
                .build(),
            Err(PlayerCantStartOnRemovedSquare {
                player: P1,
                position: pos
            })
        );

        let pos = (Col(100), Row(100));

        assert_eq!(
            SettingsBuilder::new().p1_starting(pos).build(),
            Err(PlayersMustStartOnBoard {
                player: P1,
                position: pos
            })
        );
    }

    #[test]
    fn test_you_cant_remove_and_move_to_the_same_position() {
        let mut game = GameState::new(Default::default());
        let pos = (Col(1), Row(0));

        assert_eq!(
            game.make_move(Action {
                player: P1,
                to: pos,
                remove: pos
            }),
            Err(CantRemoveTheSamePositionAsMoveTo { target: pos })
        )
    }

    #[test]
    fn test_you_cant_move_if_its_not_your_turn() {
        let mut game = GameState::new(Default::default());

        assert_eq!(
            game.make_move(Action {
                player: P2,
                to: (Col(1), Row(0)),
                remove: (Col(1), Row(1))
            }),
            Err(OtherPlayerTurn { attempted: P2 })
        )
    }

    #[test]
    fn test_you_cant_move_into_a_non_adjacent_position() {
        let mut game = SettingsBuilder::new()
            .p1_starting((Col(0), Row(0)))
            .build_game()
            .unwrap();

        let result = game.make_move(Action {
            player: game.whose_turn(),
            to: (Col(3), Row(3)),
            remove: (Col(1), Row(1)),
        });

        assert_eq!(
            result,
            Err(InvalidMoveToTarget {
                target: (Col(3), Row(3)),
                player: P1
            })
        );
    }

    #[test]
    fn test_you_cant_remove_a_position_off_the_board() {
        let target = (Col(100), Row(100));
        let game: GameState = Default::default();
        assert!(!game.is_position_allowed_to_be_removed(target, P1));
    }

    #[test]
    fn test_you_cant_remove_an_already_removed_position() {
        let remove = (Col(1), Row(1));
        let mut game = SettingsBuilder::new()
            .starting_removed_positions(vec![remove])
            .build_game()
            .unwrap();

        let action = game.valid_actions().next().unwrap();
        let result = game.make_move(Action { remove, ..action });

        assert_eq!(
            result,
            Err(InvalidRemove {
                target: (Col(1), Row(1))
            })
        );
    }

    #[test]
    fn test_when_completely_surrounded_the_game_is_over() {
        let rows = 10;
        let cols = 10;
        let p1_starting_pos = (Col(1), Row(1));
        let game = SettingsBuilder::new()
            .rows(rows)
            .cols(cols)
            .p1_starting(p1_starting_pos)
            .starting_removed_positions(
                Dimensions::new(rows, cols)
                    .unwrap()
                    .adjacenct_positions(p1_starting_pos)
                    .collect(),
            )
            .build_game()
            .unwrap();

        assert_eq!(Win { player: P2 }, game.status());
    }
}
