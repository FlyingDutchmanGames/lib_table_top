pub mod card;

use self::card::Card;
use self::card::{rank::Rank::*, suit::Suit::*};

pub type StandardDeck = [Card; 52];

pub const STANDARD_DECK: StandardDeck = [
    Card(Ace, Hearts),
    Card(King, Hearts),
    Card(Queen, Hearts),
    Card(Jack, Hearts),
    Card(Ten, Hearts),
    Card(Nine, Hearts),
    Card(Eight, Hearts),
    Card(Seven, Hearts),
    Card(Six, Hearts),
    Card(Five, Hearts),
    Card(Four, Hearts),
    Card(Three, Hearts),
    Card(Two, Hearts),
    Card(Ace, Spades),
    Card(King, Spades),
    Card(Queen, Spades),
    Card(Jack, Spades),
    Card(Ten, Spades),
    Card(Nine, Spades),
    Card(Eight, Spades),
    Card(Seven, Spades),
    Card(Six, Spades),
    Card(Five, Spades),
    Card(Four, Spades),
    Card(Three, Spades),
    Card(Two, Spades),
    Card(Ace, Diamonds),
    Card(King, Diamonds),
    Card(Queen, Diamonds),
    Card(Jack, Diamonds),
    Card(Ten, Diamonds),
    Card(Nine, Diamonds),
    Card(Eight, Diamonds),
    Card(Seven, Diamonds),
    Card(Six, Diamonds),
    Card(Five, Diamonds),
    Card(Four, Diamonds),
    Card(Three, Diamonds),
    Card(Two, Diamonds),
    Card(Ace, Clubs),
    Card(King, Clubs),
    Card(Queen, Clubs),
    Card(Jack, Clubs),
    Card(Ten, Clubs),
    Card(Nine, Clubs),
    Card(Eight, Clubs),
    Card(Seven, Clubs),
    Card(Six, Clubs),
    Card(Five, Clubs),
    Card(Four, Clubs),
    Card(Three, Clubs),
    Card(Two, Clubs),
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_standard_deck() {
        let mut unique_cards = HashSet::new();
        for card in STANDARD_DECK.iter() {
            unique_cards.insert(*card);
        }
        assert_eq!(unique_cards.len(), 52);
        assert_eq!(STANDARD_DECK.len(), 52);
    }
}
