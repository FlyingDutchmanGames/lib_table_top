use crate::common::deck::card::{rank::*, suit::*};
use enum_map::EnumMap;

pub struct Foundations(EnumMap<Suit, Option<Rank>>);

impl Foundations {
    pub fn new() -> Self {
        Self(enum_map! {_ => None})
    }

    pub fn next_for_suit(&self, suit: Suit) -> Option<Rank> {
        match self.0[suit] {
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

        for (_suit, rank) in foundations.0 {
            assert_eq!(rank, None);
        }

        for suit in Suit::ALL.iter() {
            assert_eq!(foundations.next_for_suit(*suit), Some(Rank::Ace))
        }
    }
}
