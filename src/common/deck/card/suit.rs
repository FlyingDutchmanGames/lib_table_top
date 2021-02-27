use serde::{Deserialize, Serialize};

/// The four suits of a standard deck
#[derive(
    Copy, Clone, Debug, Enum, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize, Deserialize,
)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

/// The two colors of a standard deck
#[derive(Copy, Clone, Debug, PartialEq, Eq, Enum, Hash, Serialize, Deserialize)]
pub enum Color {
    Red,
    Black,
}

use Color::*;
use Suit::*;

impl Suit {
    /// An array containing all of the suits
    /// ```
    /// use lib_table_top::common::deck::Suit::{self, *};
    ///
    /// assert_eq!([Clubs, Diamonds, Hearts, Spades], Suit::ALL);
    /// ```
    pub const ALL: [Self; 4] = [Clubs, Diamonds, Hearts, Spades];

    /// Returns the color of a suit
    /// ```
    /// use lib_table_top::common::deck::{Suit::*, Color::*};
    ///
    /// assert_eq!(Spades.color(), Black);
    /// assert_eq!(Clubs.color(), Black);
    /// assert_eq!(Diamonds.color(), Red);
    /// assert_eq!(Hearts.color(), Red);
    /// ```
    pub fn color(&self) -> Color {
        match self {
            Clubs | Spades => Black,
            Hearts | Diamonds => Red,
        }
    }
}

impl Color {
    /// Returns the suits of a color
    /// ```
    /// use lib_table_top::common::deck::{Suit::*, Color::*};
    ///
    /// assert_eq!(Red.suits(), [Diamonds, Hearts]);
    /// assert_eq!(Black.suits(), [Clubs, Spades]);
    /// ```
    pub fn suits(&self) -> [Suit; 2] {
        match self {
            Red => [Diamonds, Hearts],
            Black => [Clubs, Spades],
        }
    }
}
