use enum_map::EnumMap;

use Row::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum)]
pub enum Row {
    Row0,
    Row1,
    Row2,
}

impl Row {
    const ALL: [Self; 3] = [Row0, Row1, Row2];
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum)]
pub enum Col {
    Col0,
    Col1,
    Col2,
}

use Col::*;

impl Col {
    const ALL: [Self; 3] = [Col0, Col1, Col2];
}

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
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Marker {
    X,
    O,
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
}
