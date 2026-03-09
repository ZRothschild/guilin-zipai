use guilin_paizi_core::{
    error::GameError, game::WinResult, Card, GamePhase, GameState, Player, PlayerId,
};
use guilin_paizi_skills::SkillManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    Waiting,
    Playing,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub room_id: String,
    pub state: RoomState,
    pub players: Vec<PlayerInfo>,
    pub max_players: usize,
    pub dealer_idx: usize,
    pub current_player_idx: usize,
    pub round: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub name: String,
    pub is_ready: bool,
    pub hand_count: usize,
    pub is_online: bool,
    pub is_bot: bool,
}

pub struct GameRoom {
    pub room_id: String,
    pub state: RoomState,
    pub max_players: usize,
    pub players: Vec<Player>,
    pub ready_players: Vec<PlayerId>,
    pub game_state: GameState,
    pub skill_manager: Option<SkillManager>,
    pub player_skills: HashMap<PlayerId, u32>,
    pub online_players: HashMap<PlayerId, bool>,
}

impl GameRoom {
    pub fn new(room_id: String, max_players: usize) -> Self {
        Self {
            room_id,
            state: RoomState::Waiting,
            max_players,
            players: Vec::new(),
            ready_players: Vec::new(),
            game_state: GameState::new(),
            skill_manager: None,
            player_skills: HashMap::new(),
            online_players: HashMap::new(),
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

    pub fn add_bot(&mut self) -> Option<PlayerId> {
        if self.players.len() >= self.max_players {
            return None;
        }

        let bot_id = PlayerId::new();
        let mut bot = Player::new_bot(format!("机器人{}", self.players.len() + 1));
        bot.id = bot_id;
        self.players.push(bot);
        self.ready_players.push(bot_id);
        Some(bot_id)
    }

    pub fn remove_player(&mut self, player_id: PlayerId) -> bool {
        let len_before = self.players.len();
        self.players.retain(|p| p.id != player_id);
        self.ready_players.retain(|&id| id != player_id);
        self.online_players.remove(&player_id);
        self.players.len() < len_before
    }

    pub fn set_player_online(&mut self, player_id: PlayerId, online: bool) {
        self.online_players.insert(player_id, online);
    }

    pub fn is_player_online(&self, player_id: PlayerId) -> bool {
        self.online_players
            .get(&player_id)
            .copied()
            .unwrap_or(false)
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
        if self.state == RoomState::Playing {
            return;
        }

        for player in &self.players {
            let _ = self.game_state.add_player(player.clone());
        }

        if self.game_state.start_game().is_ok() {
            self.state = RoomState::Playing;
            self.skill_manager = Some(SkillManager::new());
        }
    }

    pub fn play_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        self.game_state.play_card(player_id, card_idx)?;
        Ok(())
    }

    pub fn chi(&mut self, player_id: PlayerId, card_indices: Vec<usize>) -> Result<(), GameError> {
        self.game_state.chi(player_id, card_indices)?;
        Ok(())
    }

    pub fn peng(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        let hand = self
            .game_state
            .hands
            .get(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if let Some(card) = hand.cards().get(card_idx) {
            self.game_state.peng(player_id, *card)?;
            Ok(())
        } else {
            Err(GameError::CardNotInHand)
        }
    }

    pub fn sao(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        let hand = self
            .game_state
            .hands
            .get(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if let Some(card) = hand.cards().get(card_idx) {
            self.game_state.sao(player_id, *card)?;
            Ok(())
        } else {
            Err(GameError::CardNotInHand)
        }
    }

    pub fn sao_chuan(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        let hand = self
            .game_state
            .hands
            .get(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if let Some(card) = hand.cards().get(card_idx) {
            self.game_state.sao_chuan(player_id, *card)?;
            Ok(())
        } else {
            Err(GameError::CardNotInHand)
        }
    }

    pub fn kai_duo(&mut self, player_id: PlayerId, card_idx: usize) -> Result<(), GameError> {
        let hand = self
            .game_state
            .hands
            .get(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if let Some(card) = hand.cards().get(card_idx) {
            self.game_state.kai_duo(player_id, *card)?;
            Ok(())
        } else {
            Err(GameError::CardNotInHand)
        }
    }

    pub fn hu(&mut self, player_id: PlayerId) -> Result<WinResult, GameError> {
        self.game_state.hu(player_id)
    }

    pub fn pass(&mut self, player_id: PlayerId) -> Result<(), GameError> {
        self.game_state.pass(player_id)
    }

    pub fn get_current_player(&self) -> Option<PlayerId> {
        if self.game_state.current_player_idx < self.game_state.players.len() {
            Some(self.game_state.players[self.game_state.current_player_idx].id)
        } else {
            None
        }
    }

    pub fn can_player_action(&self, player_id: PlayerId) -> bool {
        if let Some(current) = self.get_current_player() {
            current == player_id
        } else {
            false
        }
    }

    pub fn get_player_hand(&self, player_id: PlayerId) -> Option<&[Card]> {
        self.game_state.hands.get(&player_id).map(|h| h.cards())
    }

    pub fn get_player_hand_count(&self, player_id: PlayerId) -> usize {
        self.game_state
            .hands
            .get(&player_id)
            .map(|h| h.cards().len())
            .unwrap_or(0)
    }

    pub fn get_game_state_view(&self) -> guilin_paizi_core::game::GameStateView {
        self.game_state.to_view()
    }

    pub fn get_room_info(&self) -> RoomInfo {
        RoomInfo {
            room_id: self.room_id.clone(),
            state: self.state,
            players: self
                .players
                .iter()
                .map(|p| PlayerInfo {
                    id: p.id,
                    name: p.name.clone(),
                    is_ready: self.ready_players.contains(&p.id),
                    hand_count: self.get_player_hand_count(p.id),
                    is_online: self.is_player_online(p.id) || p.is_bot,
                    is_bot: p.is_bot,
                })
                .collect(),
            max_players: self.max_players,
            dealer_idx: self.game_state.dealer_idx,
            current_player_idx: self.game_state.current_player_idx,
            round: self.game_state.round,
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_state.phase == GamePhase::Finished
    }

    pub fn get_winner(&self) -> Option<PlayerId> {
        if let Some(result) = &self.game_state.win_result {
            Some(result.winner)
        } else {
            None
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

    pub fn get_bot_ids(&self) -> Vec<PlayerId> {
        self.players.iter().filter(|p| p.is_bot).map(|p| p.id).collect()
    }
}
