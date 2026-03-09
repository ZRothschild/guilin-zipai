use guilin_paizi_core::PlayerId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerInfo {
    pub id: PlayerId,
    pub name: String,
    pub is_ready: bool,
    pub hand_count: usize,
    pub is_online: bool,
    pub is_bot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub category: String,
    pub remaining_uses: u8,
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

impl RoomInfo {
    pub fn new(room_id: impl Into<String>) -> Self {
        Self {
            room_id: room_id.into(),
            players: Vec::new(),
            max_players: 4,
            state: RoomState::Waiting,
            dealer_idx: 0,
            current_player_idx: 0,
            round: 0,
        }
    }

    pub fn can_start(&self) -> bool {
        self.players.len() >= 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomState {
    Waiting,
    Playing,
    Finished,
}

impl RoomState {
    pub fn is_playing(&self) -> bool {
        matches!(self, RoomState::Playing)
    }
}
