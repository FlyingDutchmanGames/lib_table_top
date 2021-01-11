mod card;

use self::card::rank::*;
use self::card::suit::*;

pub struct Card(Rank, Suit);

impl Card {
    pub fn color(&self) -> Color { self.1.color() }
    pub fn suit(&self) -> Suit { self.1 }
    pub fn rank(&self) -> Rank { self.0 }
}
