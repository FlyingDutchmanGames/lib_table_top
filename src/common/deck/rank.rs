#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

pub enum Ordering {
    AceHigh,
    AceLow,
}
use Ordering::*;
use Rank::*;

impl Rank {
    pub fn next(&self, order: Ordering) -> Option<Self> {
        match (order, self) {
            (AceHigh, Ace) => None,
            (AceLow, King) => None,
            _ => Some(self.next_with_wrapping()),
        }
    }

    pub fn previous(&self, order: Ordering) -> Option<Self> {
        match (order, self) {
            (AceHigh, Two) => None,
            (AceLow, Ace) => None,
            _ => Some(self.previous_with_wrapping()),
        }
    }

    fn next_with_wrapping(&self) -> Self {
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

    fn previous_with_wrapping(&self) -> Self {
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
    fn test_rank_next_with_ace_high() {
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
            assert_eq!(test.next(AceHigh), *expected);
        }
    }

    #[test]
    fn test_rank_next_with_ace_low() {
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
            assert_eq!(test.next(AceLow), *expected);
        }
    }

    #[test]
    fn test_rank_previous_with_ace_high() {
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
            assert_eq!(test.previous(AceHigh), *expected);
        }
    }

    #[test]
    fn test_rank_previous_with_ace_low() {
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
            assert_eq!(test.previous(AceLow), *expected);
        }
    }
}
