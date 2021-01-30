use enum_map::EnumMap;
use thiserror::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Row(pub u8);
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Col(pub u8);

pub type Position = (Col, Row);

#[derive(Copy, Clone, Debug, Enum, PartialEq, Eq)]
pub enum Player {
    P1,
    P2,
}

use Player::*;

impl Player {
    fn opponent(&self) -> Self {
        match self {
            P1 => P2,
            P2 => P1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dimensions {
    rows: u8,
    cols: u8,
}

impl Dimensions {
    pub fn all_positions(&self) -> impl Iterator<Item = Position> {
        iproduct!(0..(self.cols - 1), 0..(self.rows - 1)).map(|(col, row)| (Col(col), Row(row)))
    }

    pub fn adjacenct_positions(
        &self,
        (Col(col), Row(row)): Position,
    ) -> impl Iterator<Item = Position> + '_ {
        iproduct!(
            Self::checked_adjacent(col, self.cols),
            Self::checked_adjacent(row, self.rows)
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

    pub fn valid_next_move(&self) -> Option<Action> {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
