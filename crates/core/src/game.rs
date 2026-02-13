use crate::card::Card;
use crate::deck::Deck;
use crate::error::{GameError, Result};
use crate::hand::Hand;
use crate::meld::{Meld, MeldType};
use crate::player::{Player, PlayerId, PlayerState};
use crate::constants::{DEALER_CARD_COUNT, PLAYER_CARD_COUNT, MIN_HUXI_TO_WIN, MAX_PLAYERS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Waiting,
    Dealing,
    Playing,
    Settling,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub players: Vec<Player>,
    pub hands: HashMap<PlayerId, Hand>,
    pub deck: Deck,
    pub discard_pile: Vec<(PlayerId, Card)>,
    pub current_player_idx: usize,
    pub dealer_idx: usize,
    pub round: u8,
    pub last_action: Option<GameAction>,
    pub dangdi: Option<Card>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameAction {
    PlayCard { player: PlayerId, card_idx: usize },
    Chi { player: PlayerId, cards: Vec<usize> },
    Peng { player: PlayerId, card: Card },
    Sao { player: PlayerId, card: Card },
    Hu { player: PlayerId, is_zimo: bool },
    Pass { player: PlayerId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinResult {
    pub winner: PlayerId,
    pub huxi: u8,
    pub duo: u8,
    pub fan: u8,
    pub is_zimo: bool,
    pub is_tianhu: bool,
    pub is_dihu: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::Waiting,
            players: Vec::new(),
            hands: HashMap::new(),
            deck: Deck::new(),
            discard_pile: Vec::new(),
            current_player_idx: 0,
            dealer_idx: 0,
            round: 1,
            last_action: None,
            dangdi: None,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<PlayerId> {
        if self.players.len() >= MAX_PLAYERS {
            return Err(GameError::GameFull);
        }
        let id = player.id;
        self.players.push(player);
        Ok(id)
    }

    pub fn start_game(&mut self) -> Result<()> {
        if self.players.len() < 2 {
            return Err(GameError::InvalidAction);
        }

        self.phase = GamePhase::Dealing;
        self.deck.shuffle();

        for (idx, player) in self.players.iter_mut().enumerate() {
            player.set_playing();
            player.position = idx as u8;
            
            let card_count = if idx == self.dealer_idx {
                DEALER_CARD_COUNT
            } else {
                PLAYER_CARD_COUNT
            };
            
            let cards = self.deck.draw_n(card_count);
            let mut hand = Hand::new(cards);
            hand.sort();
            
            if idx == self.dealer_idx && !hand.cards().is_empty() {
                self.dangdi = Some(hand.cards()[card_count - 1]);
            }
            
            self.hands.insert(player.id, hand);
        }

        self.phase = GamePhase::Playing;
        self.current_player_idx = self.dealer_idx;
        
        Ok(())
    }

    pub fn get_current_player(&self) -> Option<&Player> {
        self.players.get(self.current_player_idx)
    }

    pub fn get_current_player_id(&self) -> Option<PlayerId> {
        self.get_current_player().map(|p| p.id)
    }

    pub fn play_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<Card> {
        if self.phase != GamePhase::Playing {
            return Err(GameError::InvalidAction);
        }

        let current_player = self.get_current_player_id()
            .ok_or(GameError::InvalidAction)?;
        
        if player_id != current_player {
            return Err(GameError::NotYourTurn);
        }

        let hand = self.hands.get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let card = hand.remove_card(card_idx)
            .ok_or(GameError::CardNotInHand)?;

        self.discard_pile.push((player_id, card));
        
        self.draw_card(player_id)?;
        
        self.next_turn();
        
        self.last_action = Some(GameAction::PlayCard { player: player_id, card_idx });

        Ok(card)
    }

    pub fn draw_card(&mut self, player_id: PlayerId) -> Result<Option<Card>> {
        if let Some(card) = self.deck.draw() {
            if let Some(hand) = self.hands.get_mut(&player_id) {
                hand.add_card(card);
                Ok(Some(card))
            } else {
                Err(GameError::PlayerNotFound)
            }
        } else {
            Ok(None)
        }
    }

    pub fn chi(&mut self, player_id: PlayerId, card_indices: Vec<usize>) -> Result<Meld> {
        if card_indices.len() != 2 {
            return Err(GameError::InvalidMeld);
        }

        let hand = self.hands.get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let last_discard = self.discard_pile.last()
            .ok_or(GameError::InvalidAction)?;

        let mut meld_cards = vec![last_discard.1];
        
        for &idx in &card_indices {
            if idx >= hand.cards().len() {
                return Err(GameError::CardNotInHand);
            }
        }

        let indices: Vec<_> = card_indices.iter().copied().collect();
        for idx in indices.iter().rev() {
            let card = hand.cards()[*idx];
            meld_cards.push(card);
        }

        if !Meld::is_valid_chi(&meld_cards) && !Meld::is_valid_2710(&meld_cards) {
            return Err(GameError::InvalidMeld);
        }

        for idx in indices.iter().rev() {
            hand.remove_card(*idx);
        }

        let meld = Meld::new(MeldType::Chi, meld_cards, true);
        hand.add_meld(meld.clone());

        self.last_action = Some(GameAction::Chi { player: player_id, cards: card_indices });

        Ok(meld)
    }

    pub fn peng(&mut self, player_id: PlayerId, card: Card) -> Result<Meld> {
        let hand = self.hands.get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if !hand.can_peng(&card) {
            return Err(GameError::InvalidMeld);
        }

        let mut cards = vec![card, card, card];
        
        for _ in 0..2 {
            if let Some(idx) = hand.find_card(&card) {
                hand.remove_card(idx);
            }
        }

        let meld = Meld::new(MeldType::Peng, cards, true);
        hand.add_meld(meld.clone());

        self.last_action = Some(GameAction::Peng { player: player_id, card });

        Ok(meld)
    }

    pub fn calculate_hand_huxi(&self, player_id: PlayerId) -> Result<u8> {
        let hand = self.hands.get(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let mut total_huxi = 0u8;
        
        for meld in hand.melds() {
            total_huxi = total_huxi.saturating_add(meld.huxi());
        }

        Ok(total_huxi)
    }

    pub fn can_hu(&self, player_id: PlayerId) -> Result<bool> {
        let huxi = self.calculate_hand_huxi(player_id)?;
        Ok(huxi >= MIN_HUXI_TO_WIN)
    }

    fn next_turn(&mut self) {
        self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Suit, CardValue};

    #[test]
    fn test_game_creation() {
        let game = GameState::new();
        assert_eq!(game.phase, GamePhase::Waiting);
        assert!(game.players.is_empty());
    }

    #[test]
    fn test_add_player() {
        let mut game = GameState::new();
        let player = Player::new("测试");
        let id = game.add_player(player).unwrap();
        assert_eq!(game.players.len(), 1);
        assert_eq!(game.players[0].id, id);
    }

    #[test]
    fn test_game_flow() {
        let mut game = GameState::new();
        
        let p1 = Player::new("玩家1");
        let p2 = Player::new("玩家2");
        
        game.add_player(p1).unwrap();
        game.add_player(p2).unwrap();
        
        game.start_game().unwrap();
        
        assert_eq!(game.phase, GamePhase::Playing);
        assert!(game.dangdi.is_some());
    }
}
