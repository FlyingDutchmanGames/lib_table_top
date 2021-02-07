use rand::prelude::*;
use rand_chacha::ChaCha20Rng;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RngSeed(pub [u8; 32]);

impl RngSeed {
    pub fn into_rng(self) -> ChaCha20Rng {
        ChaCha20Rng::from_seed(self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn you_can_turn_a_seed_to_an_rng() {
        let seed = RngSeed([0u8; 32]);
        let mut rng = seed.into_rng();
        assert_eq!(
            [118, 160, 64],
            [rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>()]
        );
    }
}
