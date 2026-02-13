use guilin_paizi_core::{PlayerId, GameState};
use crate::EconomyConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOutcome {
    pub player_id: PlayerId,
    pub is_winner: bool,
    pub huxi: u8,
    pub duo: u8,
    pub fan: u8,
    pub is_zimo: bool,
    pub is_tianhu: bool,
    pub is_dihu: bool,
    pub skill_modifiers: Vec<SkillModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillModifier {
    pub skill_id: u32,
    pub skill_name: String,
    pub modifier_type: ModifierType,
    pub value: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModifierType {
    WinBonus,
    LossReduction,
    TargetPenalty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementResult {
    pub player_id: PlayerId,
    pub outcome: GameOutcome,
    pub base_beans: i64,
    pub skill_bonus: i64,
    pub rake_deduction: i64,
    pub final_beans: i64,
    pub rating_change: i32,
    pub new_tier: Option<String>,
}

pub struct SettlementCalculator {
    config: EconomyConfig,
}

impl SettlementCalculator {
    pub fn new(config: EconomyConfig) -> Self {
        Self { config }
    }

    pub fn calculate_settlement(
        &self,
        _game_state: &GameState,
        outcomes: Vec<GameOutcome>,
        config: &EconomyConfig,
    ) -> Vec<SettlementResult> {
        let mut results = Vec::new();
        
        let base_bet = 1000u64;
        let total_pot = base_bet * outcomes.len() as u64;
        let rake = (total_pot as f64 * config.rake_percentage) as i64;
        let distributable = total_pot as i64 - rake;

        for outcome in &outcomes {
            let (base, bonus, final_amount) = if outcome.is_winner {
                let base_win = distributable;
                let mut bonus = 0i64;
                
                for modifier in &outcome.skill_modifiers {
                    match modifier.modifier_type {
                        ModifierType::WinBonus => {
                            bonus += (base_win as f64 * modifier.value) as i64;
                        }
                        _ => {}
                    }
                }

                let multiplier = if outcome.is_zimo { 2.0 } else { 1.0 }
                    * if outcome.is_tianhu { 2.0 } else { 1.0 }
                    * if outcome.is_dihu { 2.0 } else { 1.0 };

                let final_win = ((base_win + bonus) as f64 * multiplier) as i64;
                (base_win, bonus, final_win)
            } else {
                let base_loss = -(base_bet as i64);
                let mut reduction = 0i64;
                
                for modifier in &outcome.skill_modifiers {
                    match modifier.modifier_type {
                        ModifierType::LossReduction => {
                            reduction += (base_bet as f64 * modifier.value) as i64;
                        }
                        _ => {}
                    }
                }

                let final_loss = base_loss + reduction as i64;
                (base_loss, reduction, final_loss)
            };

            results.push(SettlementResult {
                player_id: outcome.player_id,
                outcome: outcome.clone(),
                base_beans: base,
                skill_bonus: bonus,
                rake_deduction: if outcome.is_winner { 0 } else { 0 },
                final_beans: final_amount,
                rating_change: if outcome.is_winner { 15 } else { -15 },
                new_tier: None,
            });
        }

        results
    }

    pub fn calculate_duo_from_huxi(&self, huxi: u8) -> u8 {
        use guilin_paizi_core::constants::HUXI_TO_DUO;
        
        for (threshold, duo) in HUXI_TO_DUO.iter().rev() {
            if huxi >= *threshold {
                return *duo;
            }
        }
        0
    }

    pub fn calculate_fan(&self, duo: u8, is_zimo: bool, is_tianhu: bool, is_dihu: bool) -> u8 {
        let mut fan = duo;
        if is_zimo { fan += 1; }
        if is_tianhu { fan += 2; }
        if is_dihu { fan += 2; }
        fan
    }
}
