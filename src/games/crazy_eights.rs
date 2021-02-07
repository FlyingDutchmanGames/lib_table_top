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
    /// Returns the number of players for the current game type
    /// ```
    /// use lib_table_top::games::crazy_eights::GameType::*;
    ///
    /// assert_eq!(TwoPlayer.number_of_players(), 2);
    /// assert_eq!(ThreePlayer.number_of_players(), 3);
    /// assert_eq!(FourPlayer.number_of_players(), 4);
    /// ```
    pub fn number_of_players(&self) -> u8 {
        match self {
            TwoPlayer => 2,
            ThreePlayer => 3,
            FourPlayer => 4,
        }
    }

    /// Returns the starting number of cards per player
    /// ```
    /// use lib_table_top::games::crazy_eights::GameType::*;
    ///
    /// assert_eq!(TwoPlayer.starting_number_of_cards_per_player(), 7);
    /// assert_eq!(ThreePlayer.starting_number_of_cards_per_player(), 5);
    /// assert_eq!(FourPlayer.starting_number_of_cards_per_player(), 5);
    /// ```
    pub fn starting_number_of_cards_per_player(&self) -> u8 {
        match self {
            TwoPlayer => 7,
            ThreePlayer => 5,
            FourPlayer => 5,
        }
    }

    /// An iterator of players for a game type. (Players are 0 indexed)
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameType::*, Player};
    ///
    /// assert_eq!(
    ///   TwoPlayer.players().collect::<Vec<Player>>(),
    ///   vec![Player(0), Player(1)]
    /// );
    ///
    /// assert_eq!(
    ///   FourPlayer.players().collect::<Vec<Player>>(),
    ///   vec![Player(0), Player(1), Player(2), Player(3)]
    /// );
    /// ```
    pub fn players(&self) -> impl Iterator<Item = Player> + Clone {
        (0..self.number_of_players()).map(|num| Player(num))
    }
}

use GameType::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct GameHistory {
    game_type: GameType,
    seed: RngSeed,
    history: Vec<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    game_history: GameHistory,
    rng: ChaCha20Rng,
    discarded: Vec<Card>,
    hands: HashMap<Player, Vec<Card>>,
    draw_pile: Vec<Card>,
    top_card: Card,
    suit: Suit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView<'a> {
    pub player: Player,
    pub whose_turn: Player,
    pub hand: &'a [Card],
    pub discarded: &'a [Card],
    pub top_card: &'a Card,
    pub current_suit: &'a Suit,
    pub player_card_count: HashMap<Player, u8>,
    pub draw_pile_remaining: u8,
}

impl<'a> PlayerView<'a> {
    /// Returns the valid actions for a player. Player views are specific to a turn and player.
    /// There are no valid actions if it's not that player's turn
    /// ```
    /// use lib_table_top::common::deck::card::{rank::Rank::*, suit::Suit::*, Card};
    /// use lib_table_top::games::crazy_eights::{Action::*, GameState, GameType::*, Player};
    /// use lib_table_top::common::rand::RngSeed;
    ///
    /// let game = GameState::new(TwoPlayer, RngSeed([1; 32]));
    ///
    /// // If it's not that player's turn the valid actions are empty
    /// let p1 = Player(1);
    /// assert!(game.whose_turn() != p1);
    /// assert_eq!(game.player_view(p1).valid_actions(), vec![]);
    ///
    /// // The player who's turn it is has actions to take
    /// let p0 = Player(0);
    /// assert!(game.whose_turn() == p0);
    /// assert_eq!(game.player_view(p0).valid_actions(), vec![
    ///   Play(Card(Nine, Clubs)),
    ///   Play(Card(Seven, Clubs))
    /// ]);
    /// ```
    pub fn valid_actions(&self) -> Vec<Action> {
        if self.whose_turn == self.player {
            let playable: Vec<Action> = self
                .hand
                .iter()
                .flat_map(|card| match card {
                    Card(Rank::Eight, suit) => Suit::ALL
                        .iter()
                        .cloned()
                        .map(move |s| PlayEight(Card(Rank::Eight, *suit), s))
                        .collect(),
                    Card(rank, suit) if rank == &self.top_card.0 || suit == self.current_suit => {
                        vec![Play(Card(*rank, *suit))]
                    }
                    Card(_, _) => {
                        vec![]
                    }
                })
                .collect();

            if playable.is_empty() {
                vec![Draw]
            } else {
                playable
            }
        } else {
            vec![]
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Action {
    Draw,
    Play(Card),
    PlayEight(Card, Suit),
}

use Action::*;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ActionError {
    CantDrawWhenYouHavePlayableCards { player: Player, playable: Vec<Card> },
    PlayerDoesNotHaveCard { player: Player, card: Card },
    CardCantBePlayed { card: Card, top_card: Card },
}

use ActionError::*;

impl GameState {
    pub fn new(game_type: GameType, seed: RngSeed) -> Self {
        let mut rng = seed.into_rng();
        let mut cards: Vec<Card> = STANDARD_DECK.into();
        cards.shuffle(&mut rng);
        let mut deck = cards.into_iter();

        let hands: HashMap<Player, Vec<Card>> = (0..game_type.number_of_players())
            .map(|player| Player(player))
            .map(|player| {
                (
                    player,
                    (&mut deck)
                        .take(game_type.starting_number_of_cards_per_player() as usize)
                        .collect(),
                )
            })
            .collect();

        // Can't fail because deck is 52 cards
        let top_card = deck.next().unwrap();
        let draw_pile = deck.collect();

        Self {
            game_history: GameHistory {
                seed,
                game_type,
                history: Vec::new(),
            },
            rng,
            draw_pile,
            hands,
            top_card,
            suit: top_card.1,
            discarded: vec![],
        }
    }

    /// Gives the game history of the current game state, the game history is a minimal
    /// representation of the game state useful for serializing and persisting.
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, GameType::*, Player};
    /// use lib_table_top::common::rand::RngSeed;
    ///
    /// # use lib_table_top::games::crazy_eights::ActionError;
    /// // fn main() -> Result<(), ActionError> {
    /// let game = GameState::new(TwoPlayer, RngSeed([0; 32]));
    /// assert_eq!(game.game_history().game_state(), Ok(game));
    /// // }
    /// ```
    pub fn game_history(&self) -> &GameHistory {
        &self.game_history
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
        self.game_history.whose_turn()
    }

    /// Returns the view accessible to a particular player, contains all the information needed to
    /// show the game to a particular player and have them decide on their action
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, GameType::*, Player, PlayerView};
    ///
    /// use std::collections::HashMap;
    /// use lib_table_top::common::rand::RngSeed;
    /// use lib_table_top::common::deck::card::{Card, suit::Suit::*, rank::Rank::*};
    ///
    /// # use lib_table_top::games::crazy_eights::ActionError;
    /// # fn main() -> Result<(), ActionError> {
    /// let game = GameState::new(ThreePlayer, RngSeed([0; 32]));
    /// let player_view = game.player_view(Player(0));
    ///
    /// assert_eq!(player_view, PlayerView {
    ///   player: Player(0),
    ///   whose_turn: Player(0),
    ///   discarded: &[],
    ///   draw_pile_remaining: 36,
    ///   hand: &[
    ///     Card(Ace, Diamonds),
    ///     Card(Five, Spades),
    ///     Card(Two, Hearts),
    ///     Card(Jack, Diamonds),
    ///     Card(King, Spades)
    ///   ],
    ///   top_card: &Card(Four, Diamonds),
    ///   current_suit: &Diamonds,
    ///   player_card_count: vec![
    ///     (Player(0), 5u8),
    ///     (Player(1), 5u8),
    ///     (Player(2), 5u8)
    ///   ].iter().copied().collect(),
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn player_view(&self, player: Player) -> PlayerView<'_> {
        let hand: &[Card] = self
            .hands
            .get(&player)
            .map(|hand| hand.as_slice())
            .unwrap_or(&[]);
        let player_card_count: HashMap<Player, u8> = self
            .hands
            .iter()
            .map(|(player, cards)| (*player, cards.len() as u8))
            .collect();

        PlayerView {
            current_suit: &self.suit,
            discarded: self.discarded.as_slice(),
            draw_pile_remaining: self.draw_pile.len() as u8,
            hand,
            player,
            player_card_count,
            top_card: &self.top_card,
            whose_turn: self.game_history.whose_turn(),
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

        Ok(self.game_history.history.push(action))
    }

    fn player_hand(&self, player: Player) -> &[Card] {
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

impl GameHistory {
    fn new(game_type: GameType, seed: RngSeed) -> Self {
        Self {
            game_type,
            seed,
            history: Vec::new(),
        }
    }

    pub fn game_state(&self) -> Result<GameState, ActionError> {
        let mut game_state = GameState::new(self.game_type, self.seed);

        for (player, &action) in self.history() {
            game_state.make_move((player, action))?
        }

        Ok(game_state)
    }

    fn history(&self) -> impl Iterator<Item = (Player, &Action)> + '_ {
        self.history
            .iter()
            .zip((0..self.game_type.number_of_players()).cycle())
            .map(|(action, player_num)| (Player(player_num), action))
    }

    fn whose_turn(&self) -> Player {
        Player((self.history.len() as u8) % self.game_type.number_of_players())
    }

    fn undo(&mut self) -> Option<(Player, Action)> {
        let action = self.history.pop();
        action.map(|action| (self.whose_turn(), action))
    }
}
