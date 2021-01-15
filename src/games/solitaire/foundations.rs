use super::TraditionalSolitaireError;
use super::TraditionalSolitaireError::*;
use crate::common::deck::card::Card;
use crate::common::deck::card::{rank::*, suit::*};
use enum_map::EnumMap;

pub struct Foundations(EnumMap<Suit, Option<Rank>>);

impl Foundations {
    pub fn new() -> Self {
        Self(enum_map! {_ => None})
    }

    pub fn current_top_cards(&self) -> Vec<Card> {
        self.0
            .iter()
            .filter_map(|(suit, option_rank)| option_rank.map(|rank| Card(rank, suit)))
            .collect()
    }

    pub fn next_cards_needed(&self) -> Vec<Card> {
        self.0
            .iter()
            .filter_map(|(suit, option_rank)| match option_rank {
                None => Some(Card(Rank::Ace, suit)),
                Some(rank) => rank.next_with_ace_low().map(|rank| Card(rank, suit)),
            })
            .collect()
    }

    fn current_for_suit(&self, suit: Suit) -> Option<Card> {
        self.0[suit].map(|rank| Card(rank, suit))
    }

    fn next_for_suit(&self, suit: Suit) -> Option<Card> {
        match self.0[suit] {
            None => Some(Card(Rank::Ace, suit)),
            Some(rank) => rank.next_with_ace_low().map(|rank| Card(rank, suit)),
        }
    }
}

impl Foundations {
    pub fn add(&mut self, card: Card) -> Result<(), TraditionalSolitaireError> {
        let needed = self.next_for_suit(card.suit());

        if needed == Some(card) {
            self.0[card.suit()] = Some(card.rank());
            Ok(())
        } else {
            Err(CannotPlaceOnFoundation {
                attempted: card,
                needed,
            })
        }
    }

    pub fn remove(&mut self, card: Card) -> Result<(), TraditionalSolitaireError> {
        let current = self.current_for_suit(card.suit());

        if current == Some(card) {
            self.0[card.suit()] = card.rank().previous_with_ace_low();
            Ok(())
        } else {
            Err(CannotRemoveFromFoundation {
                attempted: card,
                current,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::deck::card::{rank::Rank::*, suit::Suit::*};

    #[test]
    fn test_new() {
        let foundations = Foundations::new();

        for (_suit, rank) in foundations.0 {
            assert_eq!(rank, None);
        }

        for suit in Suit::ALL.iter() {
            assert_eq!(
                foundations.next_for_suit(*suit),
                Some(Card(Rank::Ace, *suit))
            );
        }

        assert_eq!(
            foundations.next_cards_needed(),
            [
                Card(Ace, Clubs),
                Card(Ace, Diamonds),
                Card(Ace, Hearts),
                Card(Ace, Spades)
            ]
        )
    }

    #[test]
    fn add_to_foundations() {
        let mut foundations = Foundations::new();
        let card = Card(Ace, Spades);

        assert_eq!(foundations.current_top_cards(), vec![]);
        assert_eq!(foundations.add(card), Ok(()));
        assert_eq!(foundations.current_top_cards(), vec![Card(Ace, Spades)]);

        let err = foundations.add(card);

        assert_eq!(
            err,
            Err(CannotPlaceOnFoundation {
                attempted: card,
                needed: Some(Card(Two, Spades))
            })
        );
    }

    #[test]
    fn remove_from_foundations() {
        let mut foundations = Foundations::new();
        let card = Card(Ace, Spades);

        assert_eq!(
            foundations.remove(card),
            Err(CannotRemoveFromFoundation {
                attempted: card,
                current: None
            })
        );

        assert_eq!(foundations.add(card), Ok(()));
        assert_eq!(foundations.current_top_cards(), vec![card]);
        assert_eq!(foundations.remove(card), Ok(()));
        assert_eq!(foundations.current_top_cards(), vec![]);
    }
}
