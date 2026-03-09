use guilin_paizi_core::{PlayerId, GamePhase};
use crate::room::GameRoom;
use crate::message::ClientMessage;
// use rand::seq::SliceRandom;

pub struct BotLogic;

impl BotLogic {
    pub fn take_action(player_id: PlayerId, room: &GameRoom) -> Option<ClientMessage> {
        let game_state = &room.game_state;
        let room_id = room.room_id.clone();

        // 只有轮到该机器人或者有操作机会时才行动
        if game_state.phase != GamePhase::Playing {
            return None;
        }

        // 1. 检查是否可以胡牌 (最高优先级)
        if game_state.can_hu(player_id).unwrap_or(false) {
            return Some(ClientMessage::Hu { room_id });
        }

        // 2. 检查当前是否是该机器人的回合
        let is_my_turn = room.get_current_player() == Some(player_id);

        if is_my_turn {
            // 如果可以过牌（比如刚摸了一张牌而没有胡），尝试执行胡/碰/吃以外的逻辑
            // 这里简化为：如果有牌可以打，就打出的第一张
            if let Some(hand) = game_state.hands.get(&player_id) {
                if !hand.cards().is_empty() {
                    // 简单AI：打出最后一张牌
                    return Some(ClientMessage::PlayCard { 
                        room_id, 
                        card_idx: hand.cards().len() - 1 
                    });
                }
            }
        }

        // 3. 简化：如果不是我的回合，目前 AI 暂时不主动进行复杂的吃碰（除了胡）
        // 实际项目中可以增加 can_chi / can_peng 检查

        None
    }
}
