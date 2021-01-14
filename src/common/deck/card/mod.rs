pub mod rank;
pub mod suit;

use rank::*;
use suit::*;

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord)]
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
