use crate::rand::prelude::SliceRandom;
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;

use crate::common::deck::card::{rank::Rank, suit::Suit, Card};
use crate::common::deck::STANDARD_DECK;
use crate::common::rand::RngSeed;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Player(pub u8);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum GameType {
    TwoPlayer,
    ThreePlayer,
    FourPlayer,
}

impl GameType {
    pub fn number_of_players(&self) -> u8 {
        match self {
            TwoPlayer => 2,
            ThreePlayer => 3,
            FourPlayer => 4,
        }
    }

    pub fn number_of_cards_per_player(&self) -> u8 {
        match self {
            TwoPlayer => 7,
            ThreePlayer => 5,
            FourPlayer => 5,
        }
    }
}

use GameType::*;

pub struct GameState {
    game_type: GameType,
    seed: RngSeed,
    history: Vec<Action>,
}

pub struct GameView {
    game_type: GameType,
    discarded: Vec<Card>,
    hands: HashMap<Player, Vec<Card>>,
    draw_pile: Vec<Card>,
    history: Vec<Action>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Action {
    Draw,
    Play(Card),
    PlayEight(Card, Suit),
}

use Action::*;

pub enum ActionError {
    CantDrawWhenYouHavePlayableCards { player: Player, playable: Vec<Card> },
    PlayerDoesNotHaveCard { player: Player, card: Card },
    CardCantBePlayed { card: Card, needed: (Rank, Suit) },
}

use ActionError::*;

impl GameView {
    pub fn new(mut rng: ChaCha20Rng, game_type: GameType) -> Self {
        let mut cards: Vec<Card> = STANDARD_DECK.into();
        cards.shuffle(&mut rng);
        let mut deck = cards.into_iter();

        let hands: HashMap<Player, Vec<Card>> = (0..game_type.number_of_players())
            .map(|player| Player(player))
            .map(|player| {
                (
                    player,
                    (&mut deck)
                        .take(game_type.number_of_cards_per_player() as usize)
                        .collect(),
                )
            })
            .collect();

        let discarded: Vec<Card> = (&mut deck).take(1).collect();
        let draw_pile = deck.collect();

        Self {
            discarded,
            draw_pile,
            game_type,
            hands,
            history: vec![],
        }
    }

    pub fn make_move(&mut self, (player, action): (Player, Action)) -> Result<(), ActionError> {
        let (current_rank, current_suit) = self.current_rank_and_suit();
        let player_hand: &mut Vec<Card> = &mut self.hands.entry(player).or_insert_with(Vec::new);

        match action {
            Draw => {
                let playable: Vec<Card> = player_hand
                    .iter()
                    .filter(|Card(rank, suit)| {
                        rank == &Rank::Eight || rank == &current_rank || suit == &current_suit
                    })
                    .copied()
                    .collect();

                if !playable.is_empty() {
                    return Err(CantDrawWhenYouHavePlayableCards { player, playable });
                }
            }
            Play(_card) => (),
            PlayEight(_card, _suit) => (),
        }

        Ok(self.history.push(action))
    }

    pub fn current_rank_and_suit(&self) -> (Rank, Suit) {
        self.history
            .iter()
            .rev()
            .filter_map(|action| match action {
                Draw => None,
                Play(Card(rank, suit)) => Some((*rank, *suit)),
                PlayEight(_, suit) => Some((Rank::Eight, *suit)),
            })
            .next()
            .unwrap_or_else(|| {
                // It's invalid to not have any cards in the discard pile
                let Card(rank, suit) = self.discarded.iter().last().unwrap();
                (*rank, *suit)
            })
    }
}

impl GameState {
    pub fn undo(&mut self) -> Option<(Player, Action)> {
        let action = self.history.pop();
        action.map(|action| (self.whose_turn(), action))
    }
}

impl GameState {
    pub fn new(game_type: GameType, seed: RngSeed) -> GameState {
        GameState {
            game_type,
            seed,
            history: Vec::new(),
        }
    }

    pub fn history(&self) -> impl Iterator<Item = (Player, &Action)> + '_ {
        self.history
            .iter()
            .zip((0..self.game_type.number_of_players()).cycle())
            .map(|(action, player_num)| (Player(player_num), action))
    }

    pub fn game_view(&self) -> GameView {
        let rng = self.seed.into_rng();
        let game_view = GameView::new(rng, self.game_type);
        game_view
    }

    /// Gives the next player up
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, GameType::*, Player};
    /// use lib_table_top::common::rand::RngSeed;
    ///
    /// let game = GameState::new(TwoPlayer, RngSeed([0; 32]));
    /// assert_eq!(game.whose_turn(), Player(0));
    /// ```
    pub fn whose_turn(&self) -> Player {
        Player((self.history.len() as u8) % self.game_type.number_of_players())
    }
}
