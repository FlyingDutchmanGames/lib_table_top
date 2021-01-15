mod foundations;
use crate::common::deck::card::rank::Rank::*;
use crate::common::deck::card::Card;
use crate::common::deck::StandardDeck;
use enum_map::EnumMap;
use std::iter::once;
use thiserror::Error;

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

use Action::*;
use Col::*;

struct GameState {
    facedown: Tableau,
    faceup: Tableau,
    foundations: Foundations,
    stock: Vec<Card>,
    talon: Vec<Card>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    FlipCards,
    MoveCardToCol(Card, Col),
    MoveCardToFoundation(Card),
    ReloadStock,
}

impl GameState {
    pub fn new(deck: StandardDeck) -> Self {
        let faceup: Tableau = enum_map! {
            Col0 => vec!(deck[0]),
            Col1 => vec!(deck[1]),
            Col2 => vec!(deck[2]),
            Col3 => vec!(deck[3]),
            Col4 => vec!(deck[4]),
            Col5 => vec!(deck[5]),
            Col6 => vec!(deck[6]),
        };

        let facedown: Tableau = enum_map! {
            Col0 => vec!(),
            Col1 => vec!(deck[7]),
            Col2 => vec!(deck[8], deck[9]),
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
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum TraditionalSolitaireError {
    #[error("cannot flip cards when stock is empty")]
    CannotFlipWithEmptyStock,
    #[error("cannot reload stock when stock is not empty")]
    CannotReloadStockWhenStockIsNotEmpty,
    #[error("cannot move {from} to {to} because {from} must be one less rank than {to} and a different color")]
    CannotMoveCardOntoCard { from: Card, to: Card },
    #[error("can only move exposed cards (bottom of tableau or top of talon)")]
    CannotMoveNonExposedCard,
    #[error("only kings can move to empty columns, the '{attempted}' if not a king")]
    CannotMoveNonKingToEmptyCol { attempted: Card },
    #[error("can not place '{attempted}' on foundation, the needed card is '{needed:?}'")]
    CannotPlaceOnFoundation {
        attempted: Card,
        needed: Option<Card>,
    },

    #[error("can not remove '{attempted}' from foundation, the current card is '{current:?}'")]
    CannotRemoveFromFoundation {
        attempted: Card,
        current: Option<Card>,
    },
}

use TraditionalSolitaireError::*;

impl GameState {
    pub fn apply_action(&mut self, action: Action) -> Result<(), TraditionalSolitaireError> {
        match action {
            ReloadStock => self.reload_stock(),
            FlipCards => self.flip_cards(),
            MoveCardToCol(card, col) => self.move_card_to_col(card, col),
            MoveCardToFoundation(card) => self.move_card_to_foundation(card),
        }
    }

    fn move_card_to_col(&mut self, card: Card, col: Col) -> Result<(), TraditionalSolitaireError> {
        // card can be moved to spot
        let current_card: Option<&Card> = self.faceup[col].last();

        match (current_card, card.rank()) {
            (None, some_rank) if some_rank != King => {
                return Err(CannotMoveNonKingToEmptyCol { attempted: card })
            }
            (Some(current_card), _) => {
                if !can_move_card_to_card(card, *current_card) {
                    return Err(CannotMoveCardOntoCard {
                        from: card,
                        to: *current_card,
                    });
                }
            }
            _ => (),
        }

        // move the card

        todo!()
    }

    fn move_card_to_foundation(&mut self, card: Card) -> Result<(), TraditionalSolitaireError> {
        let exposed_card_column = self
            .exposed_cards()
            .iter()
            .find(|(_col, exposed_card)| *exposed_card == card)
            .map(|(col, _card)| *col);
        let is_top_of_talon = self.actionable_talon_card() == Some(card);

        if (exposed_card_column != None) || is_top_of_talon {
            self.foundations.add(card)?;
            if is_top_of_talon {
                self.talon.pop();
            }

            if let Some(col) = exposed_card_column {
                self.faceup[col].pop();
            }
            Ok(())
        } else {
            Err(CannotMoveNonExposedCard)
        }
    }

    fn reload_stock(&mut self) -> Result<(), TraditionalSolitaireError> {
        if self.stock.is_empty() {
            std::mem::swap(&mut self.stock, &mut self.talon);
            self.stock.reverse();
            Ok(())
        } else {
            Err(CannotFlipWithEmptyStock)
        }
    }

    fn flip_cards(&mut self) -> Result<(), TraditionalSolitaireError> {
        match self.stock.pop() {
            Some(card) => {
                self.talon.push(card);
                Ok(())
            }
            None => Err(CannotFlipWithEmptyStock),
        }
    }
}

fn can_move_card_to_card(card: Card, destination: Card) -> bool {
    (card.rank().next_with_ace_low() == Some(destination.rank()))
        && (card.color() != destination.color())
}

impl GameState {
    pub fn available_actions(&self) -> Vec<Action> {
        let face_up_cards = self.face_up_cards();

        let move_cards_to_exposed_cards = iproduct!(face_up_cards.clone(), self.exposed_cards())
            .filter(|(face_up_card, (_col, exposed_card))| {
                (face_up_card.color() != exposed_card.color())
                    && (face_up_card.rank().next_with_ace_low() == Some(exposed_card.rank()))
            })
            .map(|(face_up_card, (col, _exposed_card))| MoveCardToCol(face_up_card, col));

        let move_kings_to_open_columns = iproduct!(
            face_up_cards.iter().filter(|card| card.rank() == King),
            self.open_columns()
        )
        .map(|(king, col)| MoveCardToCol(*king, col));

        let move_cards_to_foundations: Vec<Action> = self
            .exposed_cards()
            .iter()
            .map(|(_col, card)| *card)
            .chain(
                self.actionable_talon_card()
                    .into_iter()
                    .collect::<Vec<Card>>(),
            )
            .filter(|card| self.foundations.next_cards_needed().contains(card))
            .map(MoveCardToFoundation)
            .collect();

        let flip_cards = if self.stock.is_empty() {
            ReloadStock
        } else {
            FlipCards
        };

        move_cards_to_exposed_cards
            .chain(move_kings_to_open_columns)
            .chain(move_cards_to_foundations)
            .chain(once(flip_cards))
            .collect()
    }

    pub fn open_columns(&self) -> Vec<Col> {
        self.faceup
            .iter()
            .filter(|(_col, cards)| cards.is_empty())
            .map(|(col, _cards)| col)
            .collect()
    }

    pub fn exposed_cards(&self) -> Vec<(Col, Card)> {
        self.faceup
            .iter()
            .filter_map(|(col, cards)| cards.last().map(|card| (col, card)))
            .map(|(col, card)| (col, *card))
            .collect()
    }

    pub fn actionable_talon_card(&self) -> Option<Card> {
        self.talon.get(0).copied()
    }

    pub fn face_up_cards(&self) -> Vec<Card> {
        self.faceup
            .iter()
            .flat_map(|(_col, cards)| cards).copied()
            .chain(self.foundations.current_top_cards())
            .chain(
                self.actionable_talon_card()
                    .into_iter()
                    .collect::<Vec<Card>>(),
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::deck::card::suit::Suit::*;
    use crate::common::deck::STANDARD_DECK;

    #[test]
    fn test_can_move_card_to_card() {
        let cards_you_can_move = [
            (Card(Ace, Spades), Card(Two, Diamonds)),
            (Card(Three, Hearts), Card(Four, Clubs)),
        ];

        for (card, destination) in cards_you_can_move.iter() {
            assert!(can_move_card_to_card(*card, *destination))
        }

        let cards_you_cant_move = [
            (Card(Ace, Spades), Card(Two, Spades)),
            (Card(Ace, Spades), Card(Two, Clubs)),
            (Card(Ace, Spades), Card(Three, Hearts)),
            (Card(King, Spades), Card(Ace, Hearts)),
        ];

        for (card, destination) in cards_you_cant_move.iter() {
            assert!(!can_move_card_to_card(*card, *destination))
        }
    }

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
