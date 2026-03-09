use crate::room::GameRoom;
use crate::message::{ClientMessage, ServerMessage};
use guilin_paizi_core::{PlayerId, GamePhase};
use guilin_paizi_skills::{SkillManager, get_skill_by_id};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MessageHandler {
    pub skill_manager: Arc<Mutex<SkillManager>>,
}

impl MessageHandler {
    pub fn new() -> Self {
        Self {
            skill_manager: Arc::new(Mutex::new(SkillManager::new())),
        }
    }

    pub async fn handle_message(
        &self,
        msg: ClientMessage,
        room: &mut GameRoom,
        player_id: PlayerId,
    ) -> Option<ServerMessage> {
        match msg {
            ClientMessage::UseSkill { room_id: _, skill_id, target } => {
                self.handle_use_skill(room, player_id, skill_id, target).await
            }
            _ => None,
        }
    }

    pub async fn handle_use_skill(
        &self,
        room: &mut GameRoom,
        player_id: PlayerId,
        skill_id: u32,
        target: Option<PlayerId>,
    ) -> Option<ServerMessage> {
        let skill = match get_skill_by_id(skill_id) {
            Some(s) => s,
            None => {
                return Some(ServerMessage::Error {
                    message: "技能不存在".to_string(),
                });
            }
        };

        let game_state = &mut room.game_state;
        
        if game_state.phase != GamePhase::Playing {
            return Some(ServerMessage::Error {
                message: "游戏未在进行中".to_string(),
            });
        }

        if !skill.can_use(game_state, player_id) {
            return Some(ServerMessage::Error {
                message: "当前无法使用该技能".to_string(),
            });
        }

        let skill_result = {
            let mut skill_box = skill;
            skill_box.use_skill(game_state, player_id, target)
        };

        if skill_result.success {
            room.player_skills.insert(player_id, skill_id);
            
            Some(ServerMessage::SkillUsed {
                player_id,
                skill_name: skill_result.message.clone(),
                effect: skill_result.effect_data.unwrap_or(serde_json::json!({})),
            })
        } else {
            Some(ServerMessage::Error {
                message: skill_result.message,
            })
        }
    }

    pub fn get_player_active_skills(&self, _player_id: PlayerId) -> Vec<u32> {
        vec![]
    }

    pub fn is_skill_active(&self, _player_id: PlayerId, _skill_id: u32) -> bool {
        false
    }
}

impl Default for MessageHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_use_skill_not_playing() {
        let handler = MessageHandler::new();
        let mut room = GameRoom::new("test_room".to_string(), 4);
        room.game_state.phase = GamePhase::Waiting;
        
        let player = Player::new("test");
        let player_id = player.id;
        
        let result = handler.handle_use_skill(&mut room, player_id, 1, None).await;
        
        if let Some(ServerMessage::Error { message }) = result {
            assert_eq!(message, "游戏未在进行中");
        }
    }

    #[tokio::test]
    async fn test_use_nonexistent_skill() {
        let handler = MessageHandler::new();
        let mut room = GameRoom::new("test_room".to_string(), 4);
        room.game_state.phase = GamePhase::Playing;
        room.game_state.add_player(Player::new("test")).unwrap();
        
        let player_id = room.game_state.players[0].id;
        
        let result = handler.handle_use_skill(&mut room, player_id, 999, None).await;
        
        if let Some(ServerMessage::Error { message }) = result {
            assert_eq!(message, "技能不存在");
        } else {
            panic!("Expected error message");
        }
    }
}
