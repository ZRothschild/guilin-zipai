use guilin_paizi_core::{GameState, PlayerId, Card};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillTrigger {
    OnTurnStart,
    OnTurnEnd,
    OnCardPlayed,
    OnMeldFormed,
    OnDrawCard,
    OnOpponentDiscard,
    OnHu,
    OnGameEnd,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    Always,
    HasCards(Vec<Card>),
    HandSize(usize),
    HuxiAbove(u8),
    InTing,
    LastRounds(u8),
    OpponentHasMeld,
    Random(f64),
}

impl TriggerCondition {
    pub fn check(&self, game_state: &GameState, player_id: PlayerId) -> bool {
        match self {
            TriggerCondition::Always => true,
            TriggerCondition::HandSize(size) => {
                if let Some(hand) = game_state.hands.get(&player_id) {
                    hand.len() == *size
                } else {
                    false
                }
            }
            TriggerCondition::HuxiAbove(threshold) => {
                game_state.calculate_hand_huxi(player_id)
                    .map(|huxi| huxi >= *threshold)
                    .unwrap_or(false)
            }
            TriggerCondition::InTing => {
                game_state.can_hu(player_id).unwrap_or(false)
            }
            TriggerCondition::LastRounds(n) => {
                let remaining = game_state.deck.remaining() as u8;
                remaining <= *n
            }
            _ => false,
        }
    }
}

pub struct TriggerContext {
    pub trigger: SkillTrigger,
    pub condition: TriggerCondition,
}

impl TriggerContext {
    pub fn new(trigger: SkillTrigger, condition: TriggerCondition) -> Self {
        Self { trigger, condition }
    }

    pub fn should_trigger(&self, game_state: &GameState, player_id: PlayerId) -> bool {
        self.condition.check(game_state, player_id)
    }
}
