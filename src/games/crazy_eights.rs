use crate::rand::prelude::SliceRandom;
use im::{HashMap, Vector};
use rand_chacha::ChaCha20Rng;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::sync::Arc;
use thiserror::Error;

use crate::common::deck::card::{rank::Rank, suit::Suit, Card};
use crate::common::deck::STANDARD_DECK;
use crate::common::rand::RngSeed;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Player(pub u8);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum NumberOfPlayers {
    Two = 2,
    Three = 3,
    Four = 4,
}

impl NumberOfPlayers {
    /// Returns the number of players for the current game type
    /// ```
    /// use lib_table_top::games::crazy_eights::NumberOfPlayers;
    ///
    /// assert_eq!(NumberOfPlayers::Two.to_int(), 2);
    /// assert_eq!(NumberOfPlayers::Three.to_int(), 3);
    /// assert_eq!(NumberOfPlayers::Four.to_int(), 4);
    /// ```
    pub fn to_int(&self) -> u8 {
        match self {
            NumberOfPlayers::Two => 2,
            NumberOfPlayers::Three => 3,
            NumberOfPlayers::Four => 4,
        }
    }

    /// Returns the starting number of cards per player
    /// ```
    /// use lib_table_top::games::crazy_eights::NumberOfPlayers::*;
    ///
    /// assert_eq!(Two.starting_number_of_cards_per_player(), 7);
    /// assert_eq!(Three.starting_number_of_cards_per_player(), 5);
    /// assert_eq!(Four.starting_number_of_cards_per_player(), 5);
    /// ```
    pub fn starting_number_of_cards_per_player(&self) -> u8 {
        match self {
            NumberOfPlayers::Two => 7,
            NumberOfPlayers::Three => 5,
            NumberOfPlayers::Four => 5,
        }
    }

    /// An iterator of players for a game type. (Players are 0 indexed)
    /// ```
    /// use lib_table_top::games::crazy_eights::{NumberOfPlayers, Player};
    ///
    /// assert_eq!(
    ///   NumberOfPlayers::Two.players().collect::<Vec<Player>>(),
    ///   vec![Player(0), Player(1)]
    /// );
    ///
    /// assert_eq!(
    ///   NumberOfPlayers::Four.players().collect::<Vec<Player>>(),
    ///   vec![Player(0), Player(1), Player(2), Player(3)]
    /// );
    /// ```
    pub fn players(&self) -> impl Iterator<Item = Player> + Clone {
        (0..self.to_int()).map(|num| Player(num))
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub seed: RngSeed,
    pub number_of_players: NumberOfPlayers,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameHistory {
    settings: Arc<Settings>,
    history: Vector<Action>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    game_history: GameHistory,
    rng: Arc<ChaCha20Rng>,
    discarded: Vector<Card>,
    hands: HashMap<Player, Vec<Card>>,
    draw_pile: Vector<Card>,
    top_card: Card,
    current_suit: Suit,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    InProgress,
    Win { player: Player },
}

use Status::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerView {
    /// The player that this player view is related to, it should only be shown to this player
    pub player: Player,
    /// The player whose turn it is, may or may not be the same as the player this view is for. If
    /// it's not the view for the player whose turn it is, that player can't make a move
    pub whose_turn: Player,
    /// The cards in this player's hand
    pub hand: Vector<Card>,
    /// The discard pile, without the "top_card" that is currently being played on
    pub discarded: Vector<Card>,
    /// The top card of the discard pile, this is the card that is next to be "played on"
    pub top_card: Card,
    /// The current suit to play, may or may not be the same as the suit of the top card, due to
    /// eights being played
    pub current_suit: Suit,
    /// Counts of the number of cards in each player's hand
    pub player_card_count: HashMap<Player, u8>,
    /// The number of cards in the draw pile
    pub draw_pile_remaining: u8,
}

impl PlayerView {
    /// Returns the valid actions for a player. Player views are specific to a turn and player.
    /// There are no valid actions if it's not that player's turn
    /// ```
    /// use lib_table_top::common::deck::card::{rank::Rank::*, suit::Suit::*, Card};
    /// use lib_table_top::games::crazy_eights::{
    ///   Action::*, GameState, NumberOfPlayers, Player, Settings
    /// };
    /// use lib_table_top::common::rand::RngSeed;
    /// use std::sync::Arc;
    ///
    /// let game = GameState::new(Arc::new(Settings { number_of_players: NumberOfPlayers::Two, seed: RngSeed([1; 32])}));
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
                    Card(rank, suit) if rank == &self.top_card.0 || suit == &self.current_suit => {
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// Draw a card from the draw pile. Reshuffles the deck if there are no cards remaining in the
    /// draw pile. If there are no cards in the draw pile or discard pile, this is a no-op.
    Draw,
    /// Play a card from your hand
    Play(Card),
    /// Play and eight, and select the next suit
    PlayEight(Card, Suit),
}

use Action::*;

#[derive(Clone, Debug, Error, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionError {
    #[error(
        "It's {:?}'s turn and not {:?}'s turn",
        correct_player,
        attempted_player
    )]
    NotPlayerTurn {
        attempted_player: Player,
        correct_player: Player,
    },
    #[error(
        "Player {:?} can't draw because they have playable cards {:?}",
        player,
        playable
    )]
    CantDrawWhenYouHavePlayableCards { player: Player, playable: Vec<Card> },
    #[error("Player {:?} does not have card {:?}", player, card)]
    PlayerDoesNotHaveCard { player: Player, card: Card },
    #[error("The Card {:?}, can not be played when the current suit is {:?} and rank is {:?}", attempted_card,current_suit, top_card.0)]
    CardCantBePlayed {
        attempted_card: Card,
        top_card: Card,
        current_suit: Suit,
    },
    #[error("Can't play the eight {:?} as a regular card", card)]
    CantPlayEightAsRegularCard { card: Card },
    #[error("Can't play {:?} as an eight", card)]
    CantPlayNonEightAsEight { card: Card },
}

use ActionError::*;

impl GameState {
    /// Creates a new game from a game type and seed
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, NumberOfPlayers, Player, Settings};
    /// use lib_table_top::common::rand::RngSeed;
    /// use std::sync::Arc;
    ///
    /// let settings = Settings {number_of_players: NumberOfPlayers::Two, seed: RngSeed([0; 32])};
    /// let game = GameState::new(Arc::new(settings));
    /// assert_eq!(game.whose_turn(), Player(0));
    /// ```
    pub fn new(settings: Arc<Settings>) -> Self {
        let mut rng = settings.seed.into_rng();
        let mut cards: Vec<Card> = STANDARD_DECK.into();
        cards.shuffle(&mut rng);
        let mut deck = cards.into_iter();

        let hands: HashMap<Player, Vec<Card>> = (0..settings.number_of_players.to_int())
            .map(Player)
            .map(|player| {
                (
                    player,
                    (&mut deck)
                        .take(
                            settings
                                .number_of_players
                                .starting_number_of_cards_per_player()
                                as usize,
                        )
                        .collect(),
                )
            })
            .collect();

        // Can't fail because deck is 52 cards
        let top_card = deck.next().unwrap();
        let draw_pile = deck.collect();

        Self {
            game_history: GameHistory {
                settings,
                history: Vector::new(),
            },
            rng: Arc::new(rng),
            draw_pile,
            hands,
            top_card,
            current_suit: top_card.1,
            discarded: Vector::new(),
        }
    }

    /// Gives the game history of the current game state, the game history is a minimal
    /// representation of the game state useful for serializing and persisting.
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, NumberOfPlayers, Player, Settings};
    /// use lib_table_top::common::rand::RngSeed;
    /// use std::sync::Arc;
    ///
    /// let settings = Settings {number_of_players: NumberOfPlayers::Two, seed: RngSeed([0; 32])};
    /// let game = GameState::new(Arc::new(settings));
    /// assert_eq!(game.game_history().game_state(), Ok(game));
    /// ```
    pub fn game_history(&self) -> &GameHistory {
        &self.game_history
    }

    /// Iterator over the actions in a game
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, NumberOfPlayers, Settings};
    /// use lib_table_top::common::rand::RngSeed;
    /// use itertools::equal;
    /// use std::sync::Arc;
    ///
    /// // A new game has an empty history
    /// let settings = Settings {number_of_players: NumberOfPlayers::Two, seed: RngSeed([0; 32])};
    /// let game = GameState::new(Arc::new(settings));
    /// assert!(equal(game.history(), vec![]));
    /// ```
    pub fn history(&self) -> impl Iterator<Item = (Player, Action)> + '_ {
        self.game_history.history()
    }

    /// Gives the next player up
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, NumberOfPlayers, Player, Settings};
    /// use lib_table_top::common::rand::RngSeed;
    /// use std::sync::Arc;
    ///
    /// let settings = Settings {number_of_players: NumberOfPlayers::Two, seed: RngSeed([0; 32])};
    /// let game = GameState::new(Arc::new(settings));
    /// assert_eq!(game.whose_turn(), Player(0));
    /// ```
    pub fn whose_turn(&self) -> Player {
        self.game_history.whose_turn()
    }

    /// Returns the player view for the current player
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, NumberOfPlayers, PlayerView, Settings};
    /// use lib_table_top::common::rand::RngSeed;
    /// use std::sync::Arc;
    ///
    /// let settings = Settings {number_of_players: NumberOfPlayers::Three, seed: RngSeed([0; 32])};
    /// let game = GameState::new(Arc::new(settings));
    /// assert_eq!(
    ///   game.player_view(game.whose_turn()),
    ///   game.current_player_view()
    /// );
    /// ```
    pub fn current_player_view(&self) -> PlayerView {
        self.player_view(self.whose_turn())
    }

    /// Returns the view accessible to a particular player, contains all the information needed to
    /// show the game to a particular player and have them decide on their action
    /// ```
    /// use lib_table_top::games::crazy_eights::{
    ///   GameState, NumberOfPlayers, Player, PlayerView, Settings
    /// };
    ///
    /// use std::collections::HashMap;
    /// use lib_table_top::common::rand::RngSeed;
    /// use lib_table_top::common::deck::card::{Card, suit::Suit::*, rank::Rank::*};
    /// use im::{Vector, vector};
    /// use std::sync::Arc;
    ///
    /// # use lib_table_top::games::crazy_eights::ActionError;
    /// # fn main() -> Result<(), ActionError> {
    /// let settings = Settings {number_of_players: NumberOfPlayers::Three, seed: RngSeed([0; 32])};
    /// let game = GameState::new(Arc::new(settings));
    /// let player_view: PlayerView = game.player_view(Player(0));
    ///
    /// assert_eq!(player_view, PlayerView {
    ///   player: Player(0),
    ///   whose_turn: Player(0),
    ///   discarded: Vector::new(),
    ///   draw_pile_remaining: 36,
    ///   hand: vector![
    ///     Card(Ace, Diamonds),
    ///     Card(Five, Spades),
    ///     Card(Two, Hearts),
    ///     Card(Jack, Diamonds),
    ///     Card(King, Spades)
    ///   ],
    ///   top_card: Card(Four, Diamonds),
    ///   current_suit: Diamonds,
    ///   player_card_count: [
    ///     (Player(0), 5u8),
    ///     (Player(1), 5u8),
    ///     (Player(2), 5u8)
    ///   ].iter().copied().collect(),
    /// });
    /// # Ok(())
    /// # }
    /// ```
    pub fn player_view(&self, player: Player) -> PlayerView {
        let hand = self
            .hands
            .get(&player)
            .map(|hand| hand.into())
            .unwrap_or(Vector::new());
        let player_card_count: HashMap<Player, u8> = self
            .hands
            .iter()
            .map(|(player, cards)| (*player, cards.len() as u8))
            .collect();

        PlayerView {
            current_suit: self.current_suit,
            discarded: self.discarded.clone(),
            draw_pile_remaining: self.draw_pile.len() as u8,
            hand,
            player,
            player_card_count,
            top_card: self.top_card,
            whose_turn: self.game_history.whose_turn(),
        }
    }

    /// Make a move on the current game, returns an error if it's illegal
    /// ```
    /// use lib_table_top::games::crazy_eights::{
    ///   GameState, NumberOfPlayers, Player, PlayerView, Action::*, ActionError::*, Settings
    /// };
    /// use lib_table_top::common::rand::RngSeed;
    /// use lib_table_top::common::deck::card::{Card, suit::Suit::*, rank::Rank::*};
    /// use std::sync::Arc;
    ///
    /// // You can play a valid action
    /// let settings = Settings {number_of_players: NumberOfPlayers::Three, seed: RngSeed([1; 32])};
    /// let mut game = GameState::new(Arc::new(settings));
    /// let action = game.current_player_view().valid_actions().pop().unwrap();
    /// assert!(game.make_move((Player(0), action)).is_ok());
    ///
    /// // Trying to play when it's not your turn is an error
    /// let err = game.make_move((Player(2), Draw));
    /// assert_eq!(
    ///   err,
    ///   Err(NotPlayerTurn { attempted_player: Player(2), correct_player: Player(1) })
    /// );
    ///
    /// assert_eq!(
    ///   &err.unwrap_err().to_string(),
    ///   "It\'s Player(1)\'s turn and not Player(2)\'s turn",
    /// );
    ///
    ///
    /// // Trying to play an eight as a regular card is illegal
    /// let err = game.make_move((Player(1), Play(Card(Eight, Spades))));
    /// assert_eq!(
    ///   err,
    ///   Err(CantPlayEightAsRegularCard { card: Card(Eight, Spades) })
    /// );
    ///
    /// assert_eq!(
    ///   &err.unwrap_err().to_string(),
    ///   "Can\'t play the eight Card(Eight, Spades) as a regular card",
    /// );
    ///
    /// // Trying to play a non eight as an eight is illegal
    /// let err = game.make_move((Player(1), PlayEight(Card(Seven, Spades), Hearts)));
    /// assert_eq!(
    ///   err,
    ///   Err(CantPlayNonEightAsEight { card: Card(Seven, Spades) })
    /// );
    ///
    /// assert_eq!(
    ///   &err.unwrap_err().to_string(),
    ///   "Can\'t play Card(Seven, Spades) as an eight",
    /// );
    ///
    /// // Trying to draw a card when you have a valid move isn't legal
    /// let err = game.make_move((Player(1), Draw));
    /// assert_eq!(
    ///   err,
    ///   Err(CantDrawWhenYouHavePlayableCards {
    ///     player: Player(1),
    ///     playable: vec![Card(Five, Spades)]
    ///   })
    /// );
    ///
    /// assert_eq!(
    ///   &err.unwrap_err().to_string(),
    ///   "Player Player(1) can\'t draw because they have playable cards [Card(Five, Spades)]",
    /// );
    ///
    /// // Trying to play a card you don't have is an error
    /// let err = game.make_move((Player(1), Play(Card(Jack, Spades))));
    /// assert_eq!(
    ///   err,
    ///   Err(PlayerDoesNotHaveCard { player: Player(1), card: Card(Jack, Spades) })
    /// );
    ///
    /// assert_eq!(
    ///   &err.unwrap_err().to_string(),
    ///   "Player Player(1) does not have card Card(Jack, Spades)",
    /// );
    ///
    /// // Trying to play a card you have but doesn't follow suit is an error
    /// let err = game.make_move((Player(1), Play(Card(Ten, Clubs))));
    /// assert_eq!(
    ///   err,
    ///   Err(CardCantBePlayed {
    ///     attempted_card: Card(Ten, Clubs),
    ///     top_card: Card(Nine, Spades),
    ///     current_suit: Spades
    ///   })
    /// );
    ///
    /// assert_eq!(
    ///   &err.unwrap_err().to_string(),
    ///   "The Card Card(Ten, Clubs), can not be played when the current suit is Spades and rank is Nine",
    /// );
    /// ```
    pub fn make_move(&mut self, (player, action): (Player, Action)) -> Result<(), ActionError> {
        self.validate_action_structure((player, action))?;

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
                    let mut new_rng = (*self.rng).clone();
                    let mut draw_pile: Vec<Card> = self
                        .draw_pile
                        .iter()
                        .chain(self.discarded.iter())
                        .copied()
                        .collect();
                    self.draw_pile.extend(self.discarded.clone());
                    draw_pile.shuffle(&mut new_rng);
                    self.draw_pile = draw_pile.into();
                    self.discarded = Vector::new();
                    self.rng = Arc::new(new_rng);
                }

                self.hands
                    .entry(player)
                    .or_insert(vec![])
                    .extend(self.draw_pile.pop_back().iter());
            }
            Play(card) => {
                self.play_card(player, card)?;
                self.current_suit = card.1;
            }
            PlayEight(card, suit) => {
                self.play_card(player, card)?;
                self.current_suit = suit;
            }
        }
        self.game_history.history.push_back(action);
        Ok(())
    }

    /// Returns the status of the game
    /// ```
    /// use lib_table_top::games::crazy_eights::{
    ///   Action, GameState, NumberOfPlayers, Status::*, Player, Settings
    /// };
    /// use lib_table_top::common::rand::RngSeed;
    /// use std::sync::Arc;
    ///
    /// let settings = Settings {number_of_players: NumberOfPlayers::Three, seed: RngSeed([1; 32])};
    /// let mut game = GameState::new(Arc::new(settings));
    /// assert_eq!(game.status(), InProgress);
    ///
    /// while InProgress == game.status() {
    ///   let action: Action = game.current_player_view().valid_actions().pop().unwrap();
    ///   let player = game.whose_turn();
    ///   assert!(game.make_move((player, action)).is_ok());
    /// }
    ///
    /// assert_eq!(game.status(), Win { player: Player(1) });
    /// ```
    pub fn status(&self) -> Status {
        self.hands
            .iter()
            .filter(|(_player, hand)| hand.is_empty())
            .map(|(&player, _hand)| Win { player })
            .next()
            .unwrap_or(InProgress)
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
                attempted_card: card,
                top_card: self.top_card,
                current_suit: self.current_suit,
            });
        }

        let old_top_card = std::mem::replace(&mut self.top_card, card);
        self.discarded.push_back(old_top_card);
        self.hands
            .entry(player)
            .or_insert(vec![])
            .retain(|c| c != &card);

        Ok(())
    }

    fn valid_to_play(&self, Card(rank, suit): &Card) -> bool {
        let Card(current_rank, _suit) = self.top_card;
        rank == &Rank::Eight || rank == &current_rank || suit == &self.current_suit
    }

    fn validate_action_structure(
        &self,
        (player, action): (Player, Action),
    ) -> Result<(), ActionError> {
        let whose_turn = self.whose_turn();
        if player != whose_turn {
            return Err(NotPlayerTurn {
                attempted_player: player,
                correct_player: whose_turn,
            });
        }

        if let Play(Card(Rank::Eight, suit)) = action {
            return Err(CantPlayEightAsRegularCard {
                card: Card(Rank::Eight, suit),
            });
        }

        if let PlayEight(Card(rank, suit), _) = action {
            if rank != Rank::Eight {
                return Err(CantPlayNonEightAsEight {
                    card: Card(rank, suit),
                });
            }
        }

        Ok(())
    }
}

impl GameHistory {
    fn new(settings: Arc<Settings>) -> Self {
        Self {
            settings,
            history: Vector::new(),
        }
    }

    /// Builds a `GameState` from the `GameHistory`, a `GameState` can be used to to make move and
    /// calculate player positions, whereas `GameHistory` is useful to serialize and persist in a
    /// smaller footprint
    /// ```
    /// use lib_table_top::games::crazy_eights::{GameState, NumberOfPlayers, Player, Settings};
    /// use lib_table_top::common::rand::RngSeed;
    /// use std::sync::Arc;
    ///
    /// let settings = Settings {number_of_players: NumberOfPlayers::Two, seed: RngSeed([1; 32])};
    /// let game = GameState::new(Arc::new(settings));
    /// assert_eq!(game.game_history().game_state(), Ok(game));
    /// ```
    pub fn game_state(&self) -> Result<GameState, ActionError> {
        let mut game_state = GameState::new(self.settings.clone());

        for (player, action) in self.history() {
            game_state.make_move((player, action))?
        }

        Ok(game_state)
    }

    fn history(&self) -> impl Iterator<Item = (Player, Action)> + '_ {
        self.history
            .iter()
            .zip(self.settings.number_of_players.players().cycle())
            .map(|(&action, player)| (player, action))
    }

    fn whose_turn(&self) -> Player {
        Player((self.history.len() as u8) % self.settings.number_of_players.to_int())
    }

    fn undo(&self) -> (Self, Option<(Player, Action)>) {
        let mut game_history = self.clone();
        let maybe_action = game_history.history.pop_back();
        let maybe_action = maybe_action.map(|action| (game_history.whose_turn(), action));
        (game_history, maybe_action)
    }
}
