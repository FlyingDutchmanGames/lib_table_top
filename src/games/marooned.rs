use enum_map::EnumMap;
use thiserror::Error;

pub struct Settings {
    rows: u8,
    cols: u8,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Row(u8);
#[derive(Debug, PartialEq, Eq)]
pub struct Col(u8);

#[derive(Debug, Enum, PartialEq, Eq)]
pub enum Player {
    One,
    Two,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum ActionError {}

pub struct GameState {
    starting_locations: EnumMap<Player, (Col, Row)>,
}

pub struct Action {
    to: (Col, Row),
    remove: (Col, Row),
}

pub enum Status {
    InProgress,
    Win { player: Player },
}

impl GameState {
    fn new() -> GameState {
        todo!()
    }

    fn status(&self) -> Status {
        todo!()
    }

    // fn removed_positions(&self) -> impl Iterator<Item=&(Col, Row)> {
    // }

    // fn available_adjacent_positions_for_player(
    //     &self,
    //     player: Player,
    // ) -> impl Iterator<Item=&(Col, Row)> {
    //     todo!()
    // }

    fn current_positions(&self) -> EnumMap<Player, (Col, Row)> {
        todo!()
    }
}
