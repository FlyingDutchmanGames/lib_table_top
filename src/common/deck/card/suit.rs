#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Red,
    Black,
}

use Color::*;
use Suit::*;

impl Suit {
    pub const ALL: [Self; 4] = [Clubs, Diamonds, Hearts, Spades];

    pub fn color(&self) -> Color {
        match self {
            Clubs | Spades => Black,
            Hearts | Diamonds => Red,
        }
    }
}

impl Color {
    pub fn suits(&self) -> [Suit; 2] {
        match self {
            Red => [Diamonds, Hearts],
            Black => [Clubs, Spades],
        }
    }
}
