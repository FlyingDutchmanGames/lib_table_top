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
    rng: ChaCha20Rng,
    game_type: GameType,
    discarded: Vec<Card>,
    hands: HashMap<Player, Vec<Card>>,
    draw_pile: Vec<Card>,
    top_card: Card,
    suit: Suit,
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
    CardCantBePlayed { card: Card, top_card: Card },
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

        // Can't fail because deck is 52 cards
        let top_card = deck.next().unwrap();
        let draw_pile = deck.collect();

        Self {
            rng,
            draw_pile,
            game_type,
            hands,
            top_card,
            suit: top_card.1,
            discarded: vec![],
        }
    }

    pub fn make_move(&mut self, (player, action): (Player, Action)) -> Result<(), ActionError> {
        match action {
            Draw => {
                let playable: Vec<Card> = self
                    .player_hand(player)
                    .iter()
                    .filter(|card| self.valid_to_play(card))
                    .copied()
                    .collect();

                if !playable.is_empty() {
                    return Err(CantDrawWhenYouHavePlayableCards { player, playable });
                }

                if self.draw_pile.is_empty() {
                    self.draw_pile.append(&mut self.discarded);
                    self.draw_pile.shuffle(&mut self.rng);
                }

                self.hands
                    .entry(player)
                    .or_insert(vec![])
                    .extend(self.draw_pile.pop().iter());
            }
            Play(card) => {
                self.play_card(player, card)?;
                self.suit = card.1;
            }
            PlayEight(card, suit) => {
                self.play_card(player, card)?;
                self.suit = suit;
            }
        }

        Ok(())
    }

    pub fn player_hand(&self, player: Player) -> &[Card] {
        &self
            .hands
            .get(&player)
            .map(|hand| hand.as_slice())
            .unwrap_or(&[])
    }

    fn play_card(&mut self, player: Player, card: Card) -> Result<(), ActionError> {
        if !self.player_hand(player).contains(&card) {
            return Err(PlayerDoesNotHaveCard { player, card });
        }

        if !self.valid_to_play(&card) {
            return Err(CardCantBePlayed {
                card,
                top_card: self.top_card,
            });
        }

        let old_top_card = std::mem::replace(&mut self.top_card, card);
        self.discarded.push(old_top_card);
        self.hands
            .entry(player)
            .or_insert(vec![])
            .retain(|c| c != &card);

        Ok(())
    }

    fn valid_to_play(&self, Card(rank, suit): &Card) -> bool {
        let Card(current_rank, _suit) = self.top_card;
        rank == &Rank::Eight || rank == &current_rank || suit == &self.suit
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
