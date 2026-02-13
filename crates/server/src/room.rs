use guilin_paizi_core::{GameState, PlayerId, Player, player::PlayerState, GamePhase, error::GameError};
use guilin_paizi_skills::SkillManager;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomState {
    Waiting,
    Playing,
    Finished,
}

pub struct GameRoom {
    pub room_id: String,
    pub state: RoomState,
    pub max_players: usize,
    pub players: Vec<Player>,
    pub ready_players: Vec<PlayerId>,
    pub game_state: Option<GameState>,
    pub skill_manager: Option<SkillManager>,
}

impl GameRoom {
    pub fn new(room_id: String, max_players: usize) -> Self {
        Self {
            room_id,
            state: RoomState::Waiting,
            max_players,
            players: Vec::new(),
            ready_players: Vec::new(),
            game_state: None,
            skill_manager: None,
        }
    }

    pub fn add_player(&mut self, player_id: PlayerId) -> bool {
        if self.players.len() >= self.max_players {
            return false;
        }
        
        if self.players.iter().any(|p| p.id == player_id) {
            return false;
        }

        let player = Player::new(format!("玩家{}", self.players.len() + 1));
        self.players.push(player);
        true
    }

    pub fn remove_player(&mut self, player_id: PlayerId) -> bool {
        let len_before = self.players.len();
        self.players.retain(|p| p.id != player_id);
        self.ready_players.retain(|&id| id != player_id);
        self.players.len() < len_before
    }

    pub fn set_player_ready(&mut self, player_id: PlayerId, ready: bool) {
        if ready {
            if !self.ready_players.contains(&player_id) {
                self.ready_players.push(player_id);
            }
        } else {
            self.ready_players.retain(|&id| id != player_id);
        }

        if self.ready_players.len() == self.players.len() && self.players.len() >= 2 {
            self.start_game();
        }
    }

    pub fn start_game(&mut self) {
        let mut game_state = GameState::new();
        
        for player in &self.players {
            let _ = game_state.add_player(player.clone());
        }

        if game_state.start_game().is_ok() {
            self.state = RoomState::Playing;
            self.game_state = Some(game_state);
            self.skill_manager = Some(SkillManager::new());
        }
    }

    pub fn play_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        if let Some(ref mut game_state) = self.game_state {
            game_state.play_card(player_id, card_idx)?;
            Ok(())
        } else {
            Err(GameError::GameNotStarted)
        }
    }

    pub fn chi(&mut self, player_id: PlayerId, card_indices: Vec<usize>) -> Result<(), GameError> {
        if let Some(ref mut game_state) = self.game_state {
            game_state.chi(player_id, card_indices)?;
            Ok(())
        } else {
            Err(GameError::GameNotStarted)
        }
    }

    pub fn peng(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        if let Some(ref mut game_state) = self.game_state {
            let hand = game_state.hands.get(&player_id)
                .ok_or(GameError::PlayerNotFound)?;
            
            if let Some(card) = hand.cards().get(card_idx) {
                game_state.peng(player_id, *card)?;
                Ok(())
            } else {
                Err(GameError::CardNotInHand)
            }
        } else {
            Err(GameError::GameNotStarted)
        }
    }

    pub fn can_start(&self) -> bool {
        self.players.len() >= 2 && self.ready_players.len() == self.players.len()
    }

    pub fn get_player_count(&self) -> usize {
        self.players.len()
    }

    pub fn is_full(&self) -> bool {
        self.players.len() >= self.max_players
    }
}
