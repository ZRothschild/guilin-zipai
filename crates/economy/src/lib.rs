use guilin_paizi_core::{PlayerId, GameState};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

    pub fn process_game_result(
        &mut self,
        game_state: &GameState,
        outcomes: Vec<GameOutcome>,
    ) -> Vec<SettlementResult> {
        self.settlement.calculate_settlement(game_state, outcomes, &self.config)
    }
}

impl Default for EconomySystem {
    fn default() -> Self {
        Self::new(EconomyConfig::default())
    }
}
