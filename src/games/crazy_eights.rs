use crate::rand::prelude::SliceRandom;
use std::collections::HashMap;

use crate::common::deck::card::{rank::Rank, suit::Suit, Card};
use crate::common::deck::STANDARD_DECK;
use crate::common::rand::RngSeed;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Player(pub u8);

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
    discard: Vec<Card>,
    hands: HashMap<Player, Vec<Card>>,
    draw_pile: Vec<Card>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Action {
    Draw,
    Play(Card),
    PlayEight(Card, Suit),
}

pub enum ActionError {
    CantDrawWhenYouHavePlayableCards,
    PlayerDoesNotHaveCard { player: Player, card: Card },
    CardCantBePlayed { card: Card, needed: (Rank, Suit) },
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
        let mut rng = self.seed.into_rng();
        let mut deck: Vec<Card> = STANDARD_DECK.into();
        deck.shuffle(&mut rng);
        let mut deck = deck.into_iter();

        let hands = GameState::deal(
            &mut deck,
            self.game_type.number_of_players(),
            self.game_type.number_of_cards_per_player(),
        );

        let discard: Vec<Card> = (&mut deck).take(1).collect();

        let gv = GameView {
            hands,
            discard,
            draw_pile: deck.collect(),
        };

        gv
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

    fn deal(
        deck: &mut dyn Iterator<Item = Card>,
        number_of_players: u8,
        number_of_cards: u8,
    ) -> HashMap<Player, Vec<Card>> {
        (0..number_of_players)
            .map(|player| Player(player))
            .map(|player| (player, deck.take(number_of_cards as usize).collect()))
            .collect()
    }
}
