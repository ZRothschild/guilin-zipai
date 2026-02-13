use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub Uuid);

impl PlayerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PlayerId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerState {
    Idle,
    Ready,
    Playing,
    Waiting,
    Finished,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub state: PlayerState,
    pub is_dealer: bool,
    pub position: u8,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: PlayerId::new(),
            name: name.into(),
            state: PlayerState::Idle,
            is_dealer: false,
            position: 0,
        }
    }

    pub fn set_ready(&mut self) {
        self.state = PlayerState::Ready;
    }

    pub fn set_playing(&mut self) {
        self.state = PlayerState::Playing;
    }

    pub fn set_dealer(&mut self, is_dealer: bool) {
        self.is_dealer = is_dealer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new("测试玩家");
        assert_eq!(player.name, "测试玩家");
        assert_eq!(player.state, PlayerState::Idle);
        assert!(!player.is_dealer);
    }

    #[test]
    fn test_player_state_transition() {
        let mut player = Player::new("测试");
        player.set_ready();
        assert_eq!(player.state, PlayerState::Ready);
        
        player.set_playing();
        assert_eq!(player.state, PlayerState::Playing);
    }
}
