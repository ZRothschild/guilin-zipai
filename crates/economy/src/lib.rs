use guilin_paizi_core::GameState;
use serde::{Deserialize, Serialize};

pub mod currency;
pub mod ranking;
pub mod settlement;

pub use currency::{CurrencySystem, HappyBeans, Transaction, TransactionType};
pub use ranking::{RankingSystem, Rank, Tier, EloRating};
pub use settlement::{SettlementCalculator, SettlementResult, GameOutcome};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub rake_percentage: f64,
    pub min_beans_for_match: u64,
    pub daily_free_beans: u64,
    pub win_bonus_base: f64,
    pub loss_reduction_skill: f64,
    pub win_bonus_skill: f64,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            rake_percentage: 0.05,
            min_beans_for_match: 1000,
            daily_free_beans: 5000,
            win_bonus_base: 1.0,
            loss_reduction_skill: 0.05,
            win_bonus_skill: 0.03,
        }
    }
}

pub struct EconomySystem {
    pub config: EconomyConfig,
    pub currency: CurrencySystem,
    pub ranking: RankingSystem,
    pub settlement: SettlementCalculator,
}

impl EconomySystem {
    pub fn new(config: EconomyConfig) -> Self {
        Self {
            config: config.clone(),
            currency: CurrencySystem::new(),
            ranking: RankingSystem::new(),
            settlement: SettlementCalculator::new(config),
        }
    }

    pub fn process_and_apply_game_result(
        &mut self,
        game_state: &GameState,
        outcomes: Vec<GameOutcome>,
    ) -> Vec<SettlementResult> {
        let results = self.settlement.calculate_settlement(game_state, outcomes, &self.config);
        
        for result in &results {
            // Apply currency change
            if let Some(beans) = self.currency.get_beans_mut(result.player_id) {
                if result.final_beans > 0 {
                    beans.add(
                        result.player_id, 
                        result.final_beans as u64, 
                        TransactionType::Win, 
                        "游戏胜利结算"
                    );
                } else if result.final_beans < 0 {
                    let amount = result.final_beans.abs() as u64;
                    beans.deduct(
                        result.player_id, 
                        amount, 
                        TransactionType::Loss, 
                        "游戏失败结算"
                    );
                }
            }

            // Apply ranking change
            self.ranking.update_rating(result.player_id, result.rating_change);
            
            // Apply rank (stars)
            if result.outcome.is_winner {
                self.ranking.update_rank(result.player_id, 1);
            } else {
                self.ranking.update_rank(result.player_id, -1);
            }
        }
        
        results
    }

    pub fn claim_daily_bonus(&mut self, player_id: guilin_paizi_core::PlayerId) -> Option<u64> {
        self.currency.claim_daily_bonus(player_id)
    }

    pub fn claim_bankruptcy_aid(&mut self, player_id: guilin_paizi_core::PlayerId) -> Result<u64, String> {
        self.currency.claim_bankruptcy_aid(player_id)
    }
}

impl Default for EconomySystem {
    fn default() -> Self {
        Self::new(EconomyConfig::default())
    }
}
