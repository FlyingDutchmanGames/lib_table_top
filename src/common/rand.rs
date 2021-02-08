use rand::prelude::*;
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RngSeed(#[serde(with = "hex")] pub [u8; 32]);

impl RngSeed {
    pub fn into_rng(self) -> ChaCha20Rng {
        ChaCha20Rng::from_seed(self.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn you_can_turn_a_seed_to_an_rng() {
        let seed = RngSeed([0u8; 32]);
        let mut rng = seed.into_rng();
        assert_eq!(
            [118, 160, 64],
            [rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>()]
        );
    }

    #[test]
    fn you_can_serialize_and_deserialize() {
        let cases = [
            (
                [0u8; 32],
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
            (
                [1u8; 32],
                "0101010101010101010101010101010101010101010101010101010101010101",
            ),
            (
                [2u8; 32],
                "0202020202020202020202020202020202020202020202020202020202020202",
            ),
            (
                [3u8; 32],
                "0303030303030303030303030303030303030303030303030303030303030303",
            ),
            (
                [4u8; 32],
                "0404040404040404040404040404040404040404040404040404040404040404",
            ),
            (
                [16u8; 32],
                "1010101010101010101010101010101010101010101010101010101010101010",
            ),
            (
                [32u8; 32],
                "2020202020202020202020202020202020202020202020202020202020202020",
            ),
            (
                [64u8; 32],
                "4040404040404040404040404040404040404040404040404040404040404040",
            ),
            (
                [128u8; 32],
                "8080808080808080808080808080808080808080808080808080808080808080",
            ),
            (
                [255u8; 32],
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            ),
        ];

        for &(bytes, hex) in cases.iter() {
            let seed = RngSeed(bytes);
            let serialized = serde_json::to_value(&seed).unwrap();
            assert_eq!(serialized, json!(hex));
            let deserialized: RngSeed = serde_json::from_value(serialized).unwrap();
            assert_eq!(&seed, &deserialized);
        }
    }
}
