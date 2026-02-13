use guilin_paizi_core::{PlayerId, Card, meld::Meld};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { token: String },
    JoinRoom { room_id: String },
    LeaveRoom { room_id: String },
    Ready { room_id: String },
    StartGame { room_id: String },
    PlayCard { room_id: String, card_idx: usize },
    Chi { room_id: String, card_indices: Vec<usize> },
    Peng { room_id: String, card_idx: usize },
    Sao { room_id: String, card_idx: usize },
    Hu { room_id: String },
    Pass { room_id: String },
    UseSkill { room_id: String, skill_id: u32, target: Option<PlayerId> },
    Chat { room_id: String, message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Welcome { player_id: PlayerId, message: String },
    Error { message: String },
    RoomJoined { room_id: String, player_id: PlayerId },
    RoomLeft { room_id: String },
    PlayerJoined { player_id: PlayerId, name: String },
    PlayerLeft { player_id: PlayerId },
    PlayerReady { player_id: PlayerId },
    GameStarted { dealer: PlayerId },
    GameStateUpdate { state: String },
    YourTurn,
    CardPlayed { player_id: PlayerId, card: Card },
    MeldFormed { player_id: PlayerId, meld: Meld },
    PlayerHu { player_id: PlayerId, is_zimo: bool },
    GameEnded { winner: Option<PlayerId> },
    SkillUsed { player_id: PlayerId, skill_name: String, effect: serde_json::Value },
    BeanUpdate { balance: u64 },
    RankUpdate { tier: String, stars: u8 },
    ChatMessage { player_id: PlayerId, message: String },
}
