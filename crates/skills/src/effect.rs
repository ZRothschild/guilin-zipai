use guilin_paizi_core::{GameState, PlayerId, Card};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillEffect {
    RevealInformation(InformationType),
    ModifyEconomy(EconomyModifier),
    AllowUndo,
    ForceAction(ForcedAction),
    BuffDebuff(BuffDebuffEffect),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InformationType {
    NextPlayerTendency,
    RecentDiscards(usize),
    RemainingCards(Card),
    DeckSize,
    OpponentHandCount(PlayerId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomyModifier {
    WinBonus(f64),
    LossReduction(f64),
    TargetPenalty { target: PlayerId, percentage: f64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForcedAction {
    MustPlayCard,
    CannotPeng,
    MustPeng(Card),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuffDebuffEffect {
    DoubleHuxi,
    ExtraDraw(u8),
    CardRevealDuration(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectResult {
    pub effect_type: String,
    pub description: String,
    pub data: Option<serde_json::Value>,
}

impl EffectResult {
    pub fn new(effect_type: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            effect_type: effect_type.into(),
            description: description.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

pub trait EffectHandler: Send + Sync {
    fn apply(&self, effect: &SkillEffect, game_state: &mut GameState, player_id: PlayerId) -> EffectResult;
}

pub struct StandardEffectHandler;

impl EffectHandler for StandardEffectHandler {
    fn apply(&self, effect: &SkillEffect, game_state: &mut GameState, player_id: PlayerId) -> EffectResult {
        match effect {
            SkillEffect::RevealInformation(info_type) => {
                match info_type {
                    InformationType::DeckSize => {
                        let size = game_state.deck.remaining();
                        EffectResult::new("明算", format!("牌堆剩余 {} 张牌", size))
                            .with_data(serde_json::json!({ "deck_size": size }))
                    }
                    InformationType::RecentDiscards(n) => {
                        let recent: Vec<_> = game_state.discard_pile.iter()
                            .rev()
                            .take(*n)
                            .map(|(_, card)| card.to_string())
                            .collect();
                        EffectResult::new("观流", format!("最近弃牌: {:?}", recent))
                            .with_data(serde_json::json!({ "discards": recent }))
                    }
                    _ => EffectResult::new("信息", "获取了隐藏信息".to_string()),
                }
            }
            SkillEffect::ModifyEconomy(modifier) => {
                match modifier {
                    EconomyModifier::WinBonus(pct) => {
                        EffectResult::new("加码", format!("胡牌时欢乐豆额外 +{:.0}%", pct * 100.0))
                            .with_data(serde_json::json!({ "bonus_percent": pct }))
                    }
                    EconomyModifier::LossReduction(pct) => {
                        EffectResult::new("稳豆", format!("输牌时欢乐豆损失减少 {:.0}%", pct * 100.0))
                            .with_data(serde_json::json!({ "reduction_percent": pct }))
                    }
                    _ => EffectResult::new("经济", "经济效果已应用".to_string()),
                }
            }
            SkillEffect::AllowUndo => {
                EffectResult::new("稳手", "出牌后2秒内可撤回".to_string())
            }
            _ => EffectResult::new("效果", "技能效果已触发".to_string()),
        }
    }
}
