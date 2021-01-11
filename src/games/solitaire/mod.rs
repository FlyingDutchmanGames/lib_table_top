mod foundations;
use crate::common::deck::card::Card;
use crate::common::deck::StandardDeck;

// https://bicyclecards.com/how-to-play/solitaire/

use foundations::Foundations;

struct GameState {
    facedown: [Vec<Card>; 7],
    faceup: [Vec<Card>; 7],
    foundations: Foundations,
    stock: Vec<Card>,
}

impl GameState {
    pub fn new(deck: StandardDeck) -> Self {
        let mut deck: Vec<Card> = deck.into();
        let facedown: [Vec<Card>; 7] =
            [0, 1, 2, 3, 4, 5, 6].map(|i| (0..i).map(|_| deck.pop().unwrap()).collect());
        let faceup: [Vec<Card>; 7] = [0; 7].map(|_| vec![deck.pop().unwrap()]);

        Self {
            foundations: Foundations::new(),
            stock: deck.into(),
            facedown,
            faceup,
        }
    }
}
