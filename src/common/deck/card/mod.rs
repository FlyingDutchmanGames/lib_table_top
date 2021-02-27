use serde::{Deserialize, Serialize};
mod rank;
mod suit;

pub use rank::Rank;
pub use suit::{Color, Suit};

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize, Deserialize)]
pub struct Card(pub Rank, pub Suit);

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} of {:?}", self.rank(), self.suit())
    }
}

impl Card {
    pub fn color(&self) -> Color {
        self.1.color()
    }
    pub fn suit(&self) -> Suit {
        self.1
    }

    pub fn rank(&self) -> Rank {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rank::Rank::*;
    use suit::Suit::*;

    #[test]
    fn test_display() {
        let test_cases = [
            (Card(Ace, Spades), "Ace of Spades"),
            (Card(King, Hearts), "King of Hearts"),
            (Card(Ten, Clubs), "Ten of Clubs"),
            (Card(Two, Diamonds), "Two of Diamonds"),
        ];

        for (card, expected) in test_cases.iter() {
            let displayed = format!("{}", card);
            assert_eq!(displayed, *expected);
        }
    }
}
