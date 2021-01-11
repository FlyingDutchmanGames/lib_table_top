use crate::common::deck::card::{rank::*, suit::*};

pub struct Foundations {
    clubs: Option<Rank>,
    diamonds: Option<Rank>,
    hearts: Option<Rank>,
    spades: Option<Rank>,
}

impl Foundations {
    pub fn new() -> Self {
        Self {
            clubs: None,
            diamonds: None,
            hearts: None,
            spades: None,
        }
    }

    pub fn next_for_suit(&self, suit: Suit) -> Option<Rank> {
        let current = {
            match suit {
                Suit::Hearts => self.hearts,
                Suit::Diamonds => self.diamonds,
                Suit::Clubs => self.clubs,
                Suit::Spades => self.spades,
            }
        };

        match current {
            None => Some(Rank::Ace),
            Some(rank) => rank.next(Ordering::AceLow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let foundations = Foundations::new();
        assert_eq!(foundations.hearts, None);
        assert_eq!(foundations.spades, None);
        assert_eq!(foundations.clubs, None);
        assert_eq!(foundations.diamonds, None);

        for suit in Suit::ALL.iter() {
            assert_eq!(foundations.next_for_suit(*suit), Some(Rank::Ace))
        }
    }
}
