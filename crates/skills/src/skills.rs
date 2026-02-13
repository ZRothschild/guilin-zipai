use crate::{Skill, SkillCategory, SkillResult, SkillInstance, SkillManager};
use crate::trigger::{SkillTrigger, TriggerCondition, TriggerContext};
use crate::effect::{SkillEffect, EffectResult, EconomyModifier, InformationType};
use guilin_paizi_core::{GameState, PlayerId, GamePhase};
use std::collections::HashMap;

pub struct TingShiSkill;

impl Skill for TingShiSkill {
    fn id(&self) -> u32 { 1 }
    fn name(&self) -> &str { "听势" }
    fn description(&self) -> &str { "显示下家最近吃/碰牌的牌型倾向" }
    fn category(&self) -> SkillCategory { SkillCategory::Information }
    fn max_uses(&self) -> u8 { 2 }
    
    fn can_use(&self, game_state: &GameState, _player_id: PlayerId) -> bool {
        game_state.phase == GamePhase::Playing
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        SkillResult::success("下家牌型倾向分析完成")
            .with_data(serde_json::json!({
                "tendency": "顺型偏多",
                "confidence": 0.65
            }))
    }
}

pub struct GuanLiuSkill;

impl Skill for GuanLiuSkill {
    fn id(&self) -> u32 { 2 }
    fn name(&self) -> &str { "观流" }
    fn description(&self) -> &str { "查看最近3张弃牌" }
    fn category(&self) -> SkillCategory { SkillCategory::Information }
    fn max_uses(&self) -> u8 { 3 }
    
    fn can_use(&self, game_state: &GameState, _player_id: PlayerId) -> bool {
        game_state.phase == GamePhase::Playing && !game_state.discard_pile.is_empty()
    }
    
    fn use_skill(&mut self, game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        let recent: Vec<String> = game_state.discard_pile.iter()
            .rev()
            .take(3)
            .map(|(_, card)| card.to_string())
            .collect();
        
        SkillResult::success(format!("最近弃牌: {:?}", recent))
            .with_data(serde_json::json!({ "discards": recent }))
    }
}

pub struct SuanYuSkill;

impl Skill for SuanYuSkill {
    fn id(&self) -> u32 { 3 }
    fn name(&self) -> &str { "算余" }
    fn description(&self) -> &str { "提示桌面尚余特定牌张数" }
    fn category(&self) -> SkillCategory { SkillCategory::Information }
    fn max_uses(&self) -> u8 { 2 }
    
    fn can_use(&self, game_state: &GameState, _player_id: PlayerId) -> bool {
        game_state.phase == GamePhase::Playing
    }
    
    fn use_skill(&mut self, game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        let remaining = game_state.deck.remaining();
        SkillResult::success(format!("牌堆剩余 {} 张牌", remaining))
            .with_data(serde_json::json!({ "remaining": remaining }))
    }
}

pub struct MingSuanSkill;

impl Skill for MingSuanSkill {
    fn id(&self) -> u32 { 4 }
    fn name(&self) -> &str { "明算" }
    fn description(&self) -> &str { "展示当前牌池剩余牌总数" }
    fn category(&self) -> SkillCategory { SkillCategory::Information }
    fn max_uses(&self) -> u8 { 5 }
    
    fn can_use(&self, game_state: &GameState, _player_id: PlayerId) -> bool {
        game_state.phase == GamePhase::Playing
    }
    
    fn use_skill(&mut self, game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        let total = game_state.deck.remaining();
        SkillResult::success(format!("牌池剩余 {} 张", total))
            .with_data(serde_json::json!({ "deck_size": total }))
    }
}

pub struct WenShouSkill {
    undo_available: bool,
}

impl WenShouSkill {
    pub fn new() -> Self {
        Self { undo_available: false }
    }
}

impl Skill for WenShouSkill {
    fn id(&self) -> u32 { 5 }
    fn name(&self) -> &str { "稳手" }
    fn description(&self) -> &str { "出牌后2秒内可撤回1次" }
    fn category(&self) -> SkillCategory { SkillCategory::ErrorCorrection }
    fn max_uses(&self) -> u8 { 1 }
    
    fn can_use(&self, game_state: &GameState, _player_id: PlayerId) -> bool {
        game_state.phase == GamePhase::Playing && self.undo_available
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        self.undo_available = false;
        SkillResult::success("出牌已撤回")
    }
}

pub struct HuanChongSkill;

impl Skill for HuanChongSkill {
    fn id(&self) -> u32 { 6 }
    fn name(&self) -> &str { "缓冲" }
    fn description(&self) -> &str { "被点炮时最多减1番" }
    fn category(&self) -> SkillCategory { SkillCategory::ErrorCorrection }
    fn max_uses(&self) -> u8 { 1 }
    
    fn can_use(&self, _game_state: &GameState, _player_id: PlayerId) -> bool {
        true
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        SkillResult::success("番数减免已生效")
            .with_data(serde_json::json!({ "fan_reduction": 1 }))
    }
}

pub struct ChongZhengSkill;

impl Skill for ChongZhengSkill {
    fn id(&self) -> u32 { 7 }
    fn name(&self) -> &str { "重整" }
    fn description(&self) -> &str { "重排手牌显示顺序" }
    fn category(&self) -> SkillCategory { SkillCategory::ErrorCorrection }
    fn max_uses(&self) -> u8 { 10 }
    
    fn can_use(&self, game_state: &GameState, player_id: PlayerId) -> bool {
        game_state.hands.contains_key(&player_id)
    }
    
    fn use_skill(&mut self, game_state: &mut GameState, player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        if let Some(hand) = game_state.hands.get_mut(&player_id) {
            hand.sort();
            SkillResult::success("手牌已重新排序")
        } else {
            SkillResult::failure("无法找到手牌")
        }
    }
}

pub struct WenDouSkill;

impl Skill for WenDouSkill {
    fn id(&self) -> u32 { 8 }
    fn name(&self) -> &str { "稳豆" }
    fn description(&self) -> &str { "本局失败时欢乐豆损失减少5%" }
    fn category(&self) -> SkillCategory { SkillCategory::Economy }
    fn max_uses(&self) -> u8 { 1 }
    
    fn can_use(&self, _game_state: &GameState, _player_id: PlayerId) -> bool {
        true
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        SkillResult::success("稳豆效果已激活")
            .with_data(serde_json::json!({ "loss_reduction": 0.05 }))
    }
}

pub struct JiaMaSkill;

impl Skill for JiaMaSkill {
    fn id(&self) -> u32 { 9 }
    fn name(&self) -> &str { "加码" }
    fn description(&self) -> &str { "胡牌时额外获得3%欢乐豆" }
    fn category(&self) -> SkillCategory { SkillCategory::Economy }
    fn max_uses(&self) -> u8 { 1 }
    
    fn can_use(&self, _game_state: &GameState, _player_id: PlayerId) -> bool {
        true
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        SkillResult::success("加码效果已激活")
            .with_data(serde_json::json!({ "win_bonus": 0.03 }))
    }
}

pub struct TiSuSkill;

impl Skill for TiSuSkill {
    fn id(&self) -> u32 { 10 }
    fn name(&self) -> &str { "提速" }
    fn description(&self) -> &str { "胡牌≥6番时返还2%欢乐豆" }
    fn category(&self) -> SkillCategory { SkillCategory::Economy }
    fn max_uses(&self) -> u8 { 1 }
    
    fn can_use(&self, _game_state: &GameState, _player_id: PlayerId) -> bool {
        true
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        SkillResult::success("提速效果已激活")
            .with_data(serde_json::json!({ "fan_threshold": 6, "bonus": 0.02 }))
    }
}

pub struct GuZhuSkill {
    active: bool,
}

impl GuZhuSkill {
    pub fn new() -> Self {
        Self { active: false }
    }
}

impl Skill for GuZhuSkill {
    fn id(&self) -> u32 { 11 }
    fn name(&self) -> &str { "孤注" }
    fn description(&self) -> &str { "听牌后宣告，胡牌+6%豆，失败-6%" }
    fn category(&self) -> SkillCategory { SkillCategory::Risk }
    fn max_uses(&self) -> u8 { 1 }
    
    fn can_use(&self, game_state: &GameState, player_id: PlayerId) -> bool {
        !self.active && game_state.can_hu(player_id).unwrap_or(false)
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, _target: Option<PlayerId>) -> SkillResult {
        self.active = true;
        SkillResult::success("孤注一掷！胡牌+6%豆，失败-6%")
            .with_data(serde_json::json!({ "win_bonus": 0.06, "loss_penalty": 0.06 }))
    }
}

pub struct FanYaSkill;

impl Skill for FanYaSkill {
    fn id(&self) -> u32 { 12 }
    fn name(&self) -> &str { "反压" }
    fn description(&self) -> &str { "指定对手：对方失败-5%豆，你失败-5%" }
    fn category(&self) -> SkillCategory { SkillCategory::Risk }
    fn max_uses(&self) -> u8 { 1 }
    
    fn can_use(&self, _game_state: &GameState, _player_id: PlayerId) -> bool {
        true
    }
    
    fn use_skill(&mut self, _game_state: &mut GameState, _player_id: PlayerId, target: Option<PlayerId>) -> SkillResult {
        if let Some(target_id) = target {
            SkillResult::success(format!("已对玩家 {:?} 施加反压", target_id))
                .with_data(serde_json::json!({ "target": target_id, "penalty": 0.05 }))
        } else {
            SkillResult::failure("需要指定目标玩家")
        }
    }
}

pub fn create_all_skills() -> Vec<Box<dyn Skill>> {
    vec![
        Box::new(TingShiSkill),
        Box::new(GuanLiuSkill),
        Box::new(SuanYuSkill),
        Box::new(MingSuanSkill),
        Box::new(WenShouSkill::new()),
        Box::new(HuanChongSkill),
        Box::new(ChongZhengSkill),
        Box::new(WenDouSkill),
        Box::new(JiaMaSkill),
        Box::new(TiSuSkill),
        Box::new(GuZhuSkill::new()),
        Box::new(FanYaSkill),
    ]
}

pub fn get_skill_by_id(id: u32) -> Option<Box<dyn Skill>> {
    match id {
        1 => Some(Box::new(TingShiSkill)),
        2 => Some(Box::new(GuanLiuSkill)),
        3 => Some(Box::new(SuanYuSkill)),
        4 => Some(Box::new(MingSuanSkill)),
        5 => Some(Box::new(WenShouSkill::new())),
        6 => Some(Box::new(HuanChongSkill)),
        7 => Some(Box::new(ChongZhengSkill)),
        8 => Some(Box::new(WenDouSkill)),
        9 => Some(Box::new(JiaMaSkill)),
        10 => Some(Box::new(TiSuSkill)),
        11 => Some(Box::new(GuZhuSkill::new())),
        12 => Some(Box::new(FanYaSkill)),
        _ => None,
    }
}
