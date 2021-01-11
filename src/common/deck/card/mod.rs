pub mod rank;
pub mod suit;

use rank::*;
use suit::*;

pub struct Card(pub Rank, pub Suit);

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
