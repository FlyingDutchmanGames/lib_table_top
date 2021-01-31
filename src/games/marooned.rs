use enum_map::EnumMap;
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Row(pub u8);
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Col(pub u8);

pub type Position = (Col, Row);

#[derive(Copy, Clone, Debug, Enum, PartialEq, Eq, PartialOrd, Ord)]
pub enum Player {
    P1,
    P2,
}

use Player::*;

impl Player {
    pub fn opponent(&self) -> Self {
        match self {
            P1 => P2,
            P2 => P1,
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum SettingsError {
    #[error("Invalid dimensions")]
    InvalidDimensions,
    #[error("Cant remove the position ({:?}) because it isn't on the board", pos)]
    CantRemovePositionNotOnBoard { pos: Position },
    #[error("Players must start at different positions")]
    PlayersCantStartAtSamePosition,
    #[error("Players must start on board, but {:?} is on {:?}", player, position)]
    PlayersMustStartOnBoard { player: Player, position: Position },
    #[error("Can't start player {:?} on removed position {:?}", player, position)]
    PlayerCantStartOnRemovedSquare { player: Player, position: Position },
}

use SettingsError::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dimensions {
    pub rows: u8,
    pub cols: u8,
}

impl Dimensions {
    pub fn new(rows: u8, cols: u8) -> Result<Self, SettingsError> {
        match (rows, cols) {
            (0, _) => Err(InvalidDimensions),
            (_, 0) => Err(InvalidDimensions),
            (1, 1) => Err(InvalidDimensions),
            _ => Ok(Self { rows, cols }),
        }
    }

    pub fn default_player_starting_positions(&self) -> EnumMap<Player, Position> {
        let col_midpoint = ((self.cols - 1) as f64) / 2f64;

        enum_map! {
            P1 => (Col(col_midpoint.ceil() as u8), Row(0)),
            P2 => (Col(col_midpoint.floor() as u8), Row(self.rows - 1)),
        }
    }

    pub fn all_positions(&self) -> impl Iterator<Item = Position> {
        iproduct!(0..(self.cols - 1), 0..(self.rows - 1)).map(|(col, row)| (Col(col), Row(row)))
    }

    pub fn is_position_on_board(&self, (Col(col), Row(row)): Position) -> bool {
        row < self.rows && col < self.cols
    }

    pub fn adjacenct_positions(
        &self,
        (Col(col), Row(row)): Position,
    ) -> impl Iterator<Item = Position> + '_ {
        iproduct!(
            Self::checked_adjacent(col, self.cols - 1),
            Self::checked_adjacent(row, self.rows - 1)
        )
        .filter(move |&(c, r)| (c, r) != (col, row))
        .map(|(c, r)| (Col(c), Row(r)))
    }

    fn checked_adjacent(starting_offset: u8, max: u8) -> Vec<u8> {
        [
            starting_offset.checked_add(1),
            Some(starting_offset),
            starting_offset.checked_sub(1),
        ]
        .iter()
        .filter_map(|offset| *offset)
        .filter(|&offset| offset < max)
        .collect()
    }
}

impl Default for Dimensions {
    fn default() -> Self {
        Self { rows: 8, cols: 6 }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    dimensions: Dimensions,
    starting_player_positions: EnumMap<Player, Position>,
    starting_removed_positions: Vec<Position>,
}

#[derive(Clone, Debug)]
pub struct SettingsBuilder {
    rows: u8,
    cols: u8,
    starting_player_positions: Option<EnumMap<Player, Position>>,
    starting_removed_positions: Vec<Position>,
}

impl Default for SettingsBuilder {
    fn default() -> Self {
        let Dimensions { rows, cols } = Default::default();
        Self {
            cols,
            rows,
            starting_player_positions: Default::default(),
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

    pub fn starting_player_positions(
        mut self,
        starting_player_positions: EnumMap<Player, Position>,
    ) -> Self {
        self.starting_player_positions = Some(starting_player_positions);
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
        let starting_player_positions = builder
            .starting_player_positions
            .unwrap_or_else(|| dimensions.default_player_starting_positions());

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Action {
    pub player: Player,
    pub to: Position,
    pub remove: Position,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    InProgress,
    Win { player: Player },
}

use Status::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub settings: Settings,
    history: Vec<Action>,
}

impl GameState {
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            history: vec![],
        }
    }

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

    pub fn whose_turn(&self) -> Player {
        self.history
            .last()
            .map(|Action { player, .. }| player.opponent())
            .unwrap_or(P1)
    }

    pub fn removed_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.settings
            .starting_removed_positions
            .iter()
            .chain(self.history.iter().map(|Action { remove, .. }| remove))
            .copied()
    }

    pub fn removable_positions(&self) -> impl Iterator<Item = Position> + '_ {
        self.settings
            .dimensions
            .all_positions()
            .filter(move |&pos| self.is_position_allowed_to_be_removed(pos))
    }

    pub fn is_position_allowed_to_be_removed(&self, position: Position) -> bool {
        (!self.removed_positions().any(|p| p == position))
            && !self
                .player_positions()
                .iter()
                .any(|(_player, pos)| *pos == position)
    }

    pub fn allowed_movement_targets_for_player(
        &self,
        player: Player,
    ) -> impl Iterator<Item = Position> + '_ {
        let removed: Vec<Position> = self.removed_positions().collect();
        let player_positions: Vec<Position> = self
            .player_positions()
            .iter()
            .map(|(_player, position)| position)
            .copied()
            .collect();

        self.settings
            .dimensions
            .adjacenct_positions(self.player_position(player))
            .filter(move |position| {
                !removed.contains(&position) && !player_positions.contains(&position)
            })
    }

    pub fn valid_next_action(&self) -> Option<Action> {
        match self.status() {
            Win { .. } => None,
            InProgress => {
                let player = self.whose_turn();
                let move_to = self.allowed_movement_targets_for_player(player).next();
                let remove = self.removable_positions().next();

                match (move_to, remove) {
                    (Some(to), Some(remove)) => Some(Action { player, to, remove }),
                    _ => None,
                }
            }
        }
    }

    fn player_positions(&self) -> EnumMap<Player, Position> {
        enum_map! {
            P1 => self.player_position(P1),
            P2 => self.player_position(P2),
        }
    }

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

        if !self.is_position_allowed_to_be_removed(action.remove) {
            return Err(InvalidRemove {
                target: action.remove,
            });
        }
        Ok(self.history.push(action))
    }

    pub fn undo(&mut self) -> Option<Action> {
        self.history.pop()
    }
}

impl GameState {
    pub fn debug_repr(&self) -> String {
        let mut debug_string: String = format!("- Who's Turn: {:?}\n\n", self.whose_turn());

        let rows = 0..(self.settings.dimensions.rows - 1);
        let cols = 0..(self.settings.dimensions.cols - 1);

        debug_string.push_str("   ");
        for col in cols.clone() {
            debug_string.push_str(&format!(" {} ", col));
        }
        debug_string.push_str("\n");

        for row in rows {
            debug_string.push_str(&format!("{} |", row));
            for col in cols.clone() {
                let position = (Col(col), Row(row));
                let marker = if self.player_position(P1) == position {
                    "1"
                } else if self.player_position(P2) == position {
                    "2"
                } else if self.removed_positions().any(|pos| pos == position) {
                    "X"
                } else {
                    "*"
                };
                debug_string.push_str(&format!(" {} ", marker));
            }
            debug_string.push_str("\n");
        }

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

        assert_eq!(
            SettingsBuilder::new()
                .starting_player_positions(
                    enum_map! { P1 => (Col(0), Row(0)), P2 => (Col(0), Row(1)) }
                )
                .starting_removed_positions(vec![(Col(0), Row(0))])
                .build(),
            Err(PlayerCantStartOnRemovedSquare {
                player: P1,
                position: (Col(0), Row(0))
            })
        );

        assert_eq!(
            SettingsBuilder::new()
                .starting_player_positions(
                    enum_map! { P1 => (Col(100), Row(100)), P2 => (Col(0), Row(0)) }
                )
                .build(),
            Err(PlayersMustStartOnBoard {
                player: P1,
                position: (Col(100), Row(100))
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
            .starting_player_positions(enum_map! { P1 => (Col(0), Row(0)), P2 => (Col(0), Row(1)) })
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
    fn test_you_cant_remove_an_already_removed_position() {
        let pos = (Col(1), Row(1));
        let mut game = SettingsBuilder::new()
            .starting_removed_positions(vec![pos])
            .build_game()
            .unwrap();

        let result = game.make_move(Action {
            remove: pos,
            ..game.valid_next_action().unwrap()
        });
        assert_eq!(
            result,
            Err(InvalidRemove {
                target: (Col(1), Row(1))
            })
        );
    }
}
