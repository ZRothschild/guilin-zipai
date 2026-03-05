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

impl PlayerProfile {
    pub fn new(nickname: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            nickname: nickname.into(),
            avatar_url: None,
            happy_beans: 10000,
            tier: "青铜".to_string(),
            stars: 1,
        }
    }
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

impl RoomInfo {
    pub fn new(room_id: impl Into<String>, host_id: impl Into<String>) -> Self {
        Self {
            room_id: room_id.into(),
            host_id: host_id.into(),
            players: Vec::new(),
            max_players: 4,
            status: RoomStatus::Waiting,
            base_bet: 100,
            skill_mode: true,
        }
    }

    pub fn add_player(&mut self, player: PlayerProfile) {
        if self.players.len() < self.max_players {
            self.players.push(player);
        }
    }

    pub fn can_start(&self) -> bool {
        self.players.len() >= 2
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomStatus {
    Waiting,
    Playing,
    Finished,
}

impl RoomStatus {
    pub fn is_playing(&self) -> bool {
        matches!(self, RoomStatus::Playing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_profile_creation() {
        let player = PlayerProfile::new("测试玩家");
        assert_eq!(player.nickname, "测试玩家");
        assert_eq!(player.happy_beans, 10000);
        assert_eq!(player.tier, "青铜");
    }

    #[test]
    fn test_room_info_creation() {
        let room = RoomInfo::new("room_1", "host_1");
        assert_eq!(room.room_id, "room_1");
        assert_eq!(room.status, RoomStatus::Waiting);
        assert!(room.skill_mode);
    }

    #[test]
    fn test_room_add_player() {
        let mut room = RoomInfo::new("room_1", "host_1");
        room.add_player(PlayerProfile::new("玩家1"));
        room.add_player(PlayerProfile::new("玩家2"));

        assert_eq!(room.players.len(), 2);
        assert!(room.can_start());
    }

    #[test]
    fn test_room_status() {
        let waiting = RoomStatus::Waiting;
        let playing = RoomStatus::Playing;

        assert!(!waiting.is_playing());
        assert!(playing.is_playing());
    }
}
