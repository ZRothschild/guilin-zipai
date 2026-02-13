use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub id: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub happy_beans: u64,
    pub tier: String,
    pub stars: u8,
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
    pub host_id: String,
    pub players: Vec<PlayerProfile>,
    pub max_players: usize,
    pub status: RoomStatus,
    pub base_bet: u64,
    pub skill_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomStatus {
    Waiting,
    Playing,
    Finished,
}
