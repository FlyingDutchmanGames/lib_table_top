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
    talon: Vec<Card>,
}

pub enum Action {
    FlipCard,
    MoveCardFromFoundation(Card, Col),
    MoveCardToCol(Card, Col),
    MoveCardToFoundation(Card),
    MoveCardToOpenColumn(Card, Col),
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
            talon: vec![],
            facedown,
            faceup,
        }
    }

    pub fn available_actions(&self) -> Vec<Action> {
        todo!()
    }

    pub fn open_columns(&self) -> Vec<Col> {
        self.faceup
            .iter()
            .filter(|(_col, cards)| cards.is_empty())
            .map(|(col, _cards)| col)
            .collect()
    }

    pub fn exposed_cards(&self) -> Vec<Card> {
        self.faceup
            .iter()
            .filter_map(|(_col, cards)| cards.get(0))
            .map(|card| *card)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::deck::card::{rank::Rank::*, suit::Suit::*};
    use crate::common::deck::STANDARD_DECK;

    #[test]
    fn test_game_state_new() {
        let mut deck = STANDARD_DECK;
        deck.sort();
        let gs = GameState::new(deck);

        let mut num_cards = gs.stock.len();

        for (_col, faceup) in &gs.faceup {
            num_cards += faceup.len();
        }

        for (_col, facedown) in &gs.facedown {
            num_cards += facedown.len();
        }

        assert_eq!(num_cards, 52);

        // face up cards
        assert_eq!(gs.faceup[Col0], vec!(Card(Ace, Clubs)));
        assert_eq!(gs.faceup[Col1], vec!(Card(Ace, Diamonds)));
        assert_eq!(gs.faceup[Col2], vec!(Card(Ace, Hearts)));
        assert_eq!(gs.faceup[Col3], vec!(Card(Ace, Spades)));
        assert_eq!(gs.faceup[Col4], vec!(Card(Two, Clubs)));
        assert_eq!(gs.faceup[Col5], vec!(Card(Two, Diamonds)));
        assert_eq!(gs.faceup[Col6], vec!(Card(Two, Hearts)));

        // face down cards
        assert_eq!(gs.facedown[Col0], vec!());
        assert_eq!(gs.facedown[Col1], vec!(Card(Two, Spades)));
        assert_eq!(
            gs.facedown[Col2],
            vec!(Card(Three, Clubs), Card(Three, Diamonds))
        );
        assert_eq!(
            gs.facedown[Col3],
            vec!(Card(Three, Hearts), Card(Three, Spades), Card(Four, Clubs))
        );
        assert_eq!(
            gs.facedown[Col4],
            vec!(
                Card(Four, Diamonds),
                Card(Four, Hearts),
                Card(Four, Spades),
                Card(Five, Clubs)
            )
        );
        assert_eq!(
            gs.facedown[Col5],
            vec!(
                Card(Five, Diamonds),
                Card(Five, Hearts),
                Card(Five, Spades),
                Card(Six, Clubs),
                Card(Six, Diamonds)
            )
        );
        assert_eq!(
            gs.facedown[Col6],
            vec!(
                Card(Six, Hearts),
                Card(Six, Spades),
                Card(Seven, Clubs),
                Card(Seven, Diamonds),
                Card(Seven, Hearts),
                Card(Seven, Spades)
            )
        );
    }
}
