mod foundations;
use crate::common::deck::card::Card;
use crate::common::deck::StandardDeck;
use enum_map::EnumMap;

// https://bicyclecards.com/how-to-play/solitaire/

use foundations::Foundations;

type Tableau = EnumMap<Col, Vec<Card>>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum, Hash)]
pub enum Col {
    Col0,
    Col1,
    Col2,
    Col3,
    Col4,
    Col5,
    Col6,
}

use Col::*;

struct GameState {
    facedown: Tableau,
    faceup: Tableau,
    foundations: Foundations,
    stock: Vec<Card>,
}

impl GameState {
    pub fn new(deck: StandardDeck) -> Self {
        let faceup: Tableau = enum_map! {
            Col0 => vec!(deck[00]),
            Col1 => vec!(deck[01]),
            Col2 => vec!(deck[02]),
            Col3 => vec!(deck[03]),
            Col4 => vec!(deck[04]),
            Col5 => vec!(deck[05]),
            Col6 => vec!(deck[06]),
        };

        let facedown: Tableau = enum_map! {
            Col0 => vec!(),
            Col1 => vec!(deck[07]),
            Col2 => vec!(deck[08], deck[09]),
            Col3 => vec!(deck[10], deck[11], deck[12]),
            Col4 => vec!(deck[13], deck[14], deck[15], deck[16]),
            Col5 => vec!(deck[17], deck[18], deck[19], deck[20], deck[21]),
            Col6 => vec!(deck[22], deck[23], deck[24], deck[25], deck[26], deck[27]),
        };

        Self {
            foundations: Foundations::new(),
            stock: deck[28..].into(),
            facedown,
            faceup,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::deck::STANDARD_DECK;

    #[test]
    fn test_game_state_new() {
        let gs = GameState::new(STANDARD_DECK);

        let mut num_cards = gs.stock.len();

        for (_col, faceup) in gs.faceup {
            num_cards += faceup.len();
        }

        for (_col, facedown) in gs.facedown {
            num_cards += facedown.len();
        }

        assert_eq!(num_cards, 52)
    }
}
