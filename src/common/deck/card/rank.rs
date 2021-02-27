use serde_repr::*;

/// The pips of a standard deck. Important note that the cards have `repr(u8)` and Ace is
/// represented by 1
#[derive(
    Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Hash, Ord, Serialize_repr, Deserialize_repr,
)]
#[repr(u8)]
pub enum Rank {
    Ace = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
}

use Rank::*;

impl Rank {
    pub const ALL: [Self; 13] = [
        Ace, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King,
    ];

    /// Returns the next card, with Ace being high
    /// ```
    /// use lib_table_top::common::deck::Rank::*;
    ///
    /// assert_eq!(Ace.next_with_ace_high(), None);
    /// assert_eq!(King.next_with_ace_high(), Some(Ace));
    /// ```
    pub fn next_with_ace_high(&self) -> Option<Self> {
        match self {
            Ace => None,
            _ => Some(self.next_with_wrapping()),
        }
    }

    /// Returns the next card, with Ace being low
    /// ```
    /// use lib_table_top::common::deck::Rank::*;
    ///
    /// assert_eq!(King.next_with_ace_low(), None);
    /// assert_eq!(Ace.next_with_ace_low(), Some(Two));
    /// ```
    pub fn next_with_ace_low(&self) -> Option<Self> {
        match self {
            King => None,
            _ => Some(self.next_with_wrapping()),
        }
    }

    /// Returns the previous card, with Ace being high
    /// ```
    /// use lib_table_top::common::deck::Rank::*;
    ///
    /// assert_eq!(Two.previous_with_ace_high(), None);
    /// assert_eq!(Ace.previous_with_ace_high(), Some(King));
    /// ```
    pub fn previous_with_ace_high(&self) -> Option<Self> {
        match self {
            Two => None,
            _ => Some(self.previous_with_wrapping()),
        }
    }

    /// Returns the previous card, with Ace being high
    /// ```
    /// use lib_table_top::common::deck::Rank::*;
    ///
    /// assert_eq!(Two.previous_with_ace_low(), Some(Ace));
    /// assert_eq!(Ace.previous_with_ace_low(), None);
    /// ```
    pub fn previous_with_ace_low(&self) -> Option<Self> {
        match self {
            Ace => None,
            _ => Some(self.previous_with_wrapping()),
        }
    }

    /// Provides the next highest card, wraps from King => Ace => Two
    /// ```
    /// use lib_table_top::common::deck::Rank::*;
    ///
    /// assert_eq!(King.next_with_wrapping(), Ace);
    /// assert_eq!(Ace.next_with_wrapping(), Two);
    /// assert_eq!(Two.next_with_wrapping(), Three);
    /// // etc ..
    /// ```
    pub fn next_with_wrapping(&self) -> Self {
        match self {
            Ace => Two,
            Two => Three,
            Three => Four,
            Four => Five,
            Five => Six,
            Six => Seven,
            Seven => Eight,
            Eight => Nine,
            Nine => Ten,
            Ten => Jack,
            Jack => Queen,
            Queen => King,
            King => Ace,
        }
    }

    /// Provides the next lowest card, wraps from Two => Ace => King
    /// ```
    /// use lib_table_top::common::deck::Rank::*;
    ///
    /// assert_eq!(Two.previous_with_wrapping(), Ace);
    /// assert_eq!(Ace.previous_with_wrapping(), King);
    /// assert_eq!(King.previous_with_wrapping(), Queen);
    /// // etc ..
    /// ```
    pub fn previous_with_wrapping(&self) -> Self {
        match self {
            Ace => King,
            King => Queen,
            Queen => Jack,
            Jack => Ten,
            Ten => Nine,
            Nine => Eight,
            Eight => Seven,
            Seven => Six,
            Six => Five,
            Five => Four,
            Four => Three,
            Three => Two,
            Two => Ace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_with_ace_high() {
        let test_cases = [
            (Ace, None),
            (King, Some(Ace)),
            (Queen, Some(King)),
            (Jack, Some(Queen)),
            (Ten, Some(Jack)),
            (Nine, Some(Ten)),
            (Eight, Some(Nine)),
            (Seven, Some(Eight)),
            (Six, Some(Seven)),
            (Five, Some(Six)),
            (Four, Some(Five)),
            (Three, Some(Four)),
            (Two, Some(Three)),
        ];

        for (test, expected) in test_cases.iter() {
            assert_eq!(test.next_with_ace_high(), *expected);
        }
    }

    #[test]
    fn test_next_with_ace_low() {
        let test_cases = [
            (King, None),
            (Queen, Some(King)),
            (Jack, Some(Queen)),
            (Ten, Some(Jack)),
            (Nine, Some(Ten)),
            (Eight, Some(Nine)),
            (Seven, Some(Eight)),
            (Six, Some(Seven)),
            (Five, Some(Six)),
            (Four, Some(Five)),
            (Three, Some(Four)),
            (Two, Some(Three)),
            (Ace, Some(Two)),
        ];

        for (test, expected) in test_cases.iter() {
            assert_eq!(test.next_with_ace_low(), *expected);
        }
    }

    #[test]
    fn test_previous_with_ace_high() {
        let test_cases = [
            (Ace, Some(King)),
            (King, Some(Queen)),
            (Queen, Some(Jack)),
            (Jack, Some(Ten)),
            (Ten, Some(Nine)),
            (Nine, Some(Eight)),
            (Eight, Some(Seven)),
            (Seven, Some(Six)),
            (Six, Some(Five)),
            (Five, Some(Four)),
            (Four, Some(Three)),
            (Three, Some(Two)),
            (Two, None),
        ];

        for (test, expected) in test_cases.iter() {
            assert_eq!(test.previous_with_ace_high(), *expected);
        }
    }

    #[test]
    fn test_previous_with_ace_low() {
        let test_cases = [
            (King, Some(Queen)),
            (Queen, Some(Jack)),
            (Jack, Some(Ten)),
            (Ten, Some(Nine)),
            (Nine, Some(Eight)),
            (Eight, Some(Seven)),
            (Seven, Some(Six)),
            (Six, Some(Five)),
            (Five, Some(Four)),
            (Four, Some(Three)),
            (Three, Some(Two)),
            (Two, Some(Ace)),
            (Ace, None),
        ];

        for (test, expected) in test_cases.iter() {
            assert_eq!(test.previous_with_ace_low(), *expected);
        }
    }
}
