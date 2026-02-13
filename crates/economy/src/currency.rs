use guilin_paizi_core::PlayerId;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Win,
    Loss,
    EntryFee,
    Rake,
    DailyBonus,
    SkillBonus,
    SkillReduction,
    Purchase,
    Refund,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub player_id: PlayerId,
    pub amount: i64,
    pub transaction_type: TransactionType,
    pub timestamp: DateTime<Utc>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HappyBeans {
    pub balance: u64,
    pub total_earned: u64,
    pub total_spent: u64,
    pub transaction_history: Vec<Transaction>,
}

impl HappyBeans {
    pub fn new(initial_balance: u64) -> Self {
        Self {
            balance: initial_balance,
            total_earned: initial_balance,
            total_spent: 0,
            transaction_history: Vec::new(),
        }
    }

    pub fn add(&mut self, amount: u64, transaction_type: TransactionType, description: impl Into<String>) {
        self.balance += amount;
        self.total_earned += amount;
        self.transaction_history.push(Transaction {
            player_id: PlayerId::new(),
            amount: amount as i64,
            transaction_type,
            timestamp: Utc::now(),
            description: description.into(),
        });
    }

    pub fn deduct(&mut self, amount: u64, transaction_type: TransactionType, description: impl Into<String>) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            self.total_spent += amount;
            self.transaction_history.push(Transaction {
                player_id: PlayerId::new(),
                amount: -(amount as i64),
                transaction_type,
                timestamp: Utc::now(),
                description: description.into(),
            });
            true
        } else {
            false
        }
    }

    pub fn has_sufficient(&self, amount: u64) -> bool {
        self.balance >= amount
    }
}

pub struct CurrencySystem {
    player_balances: HashMap<PlayerId, HappyBeans>,
    daily_claims: HashMap<PlayerId, DateTime<Utc>>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        Self {
            player_balances: HashMap::new(),
            daily_claims: HashMap::new(),
        }
    }

    pub fn register_player(&mut self, player_id: PlayerId, initial_balance: u64) {
        self.player_balances.insert(player_id, HappyBeans::new(initial_balance));
    }

    pub fn get_balance(&self, player_id: PlayerId) -> Option<u64> {
        self.player_balances.get(&player_id).map(|b| b.balance)
    }

    pub fn get_beans(&self, player_id: PlayerId) -> Option<&HappyBeans> {
        self.player_balances.get(&player_id)
    }

    pub fn get_beans_mut(&mut self, player_id: PlayerId) -> Option<&mut HappyBeans> {
        self.player_balances.get_mut(&player_id)
    }

    pub fn can_claim_daily(&self, player_id: PlayerId) -> bool {
        match self.daily_claims.get(&player_id) {
            None => true,
            Some(last_claim) => {
                let now = Utc::now();
                let days_since = now.signed_duration_since(*last_claim).num_days();
                days_since >= 1
            }
        }
    }

    pub fn claim_daily_bonus(&mut self, player_id: PlayerId, amount: u64) -> Option<u64> {
        if self.can_claim_daily(player_id) {
            if let Some(beans) = self.player_balances.get_mut(&player_id) {
                beans.add(amount, TransactionType::DailyBonus, "每日签到奖励");
                self.daily_claims.insert(player_id, Utc::now());
                return Some(beans.balance);
            }
        }
        None
    }

    pub fn transfer(&mut self, from: PlayerId, to: PlayerId, amount: u64, description: impl Into<String>) -> bool {
        if let Some(from_beans) = self.player_balances.get_mut(&from) {
            if from_beans.deduct(amount, TransactionType::Loss, description) {
                if let Some(to_beans) = self.player_balances.get_mut(&to) {
                    to_beans.add(amount, TransactionType::Win, "赢得游戏");
                    return true;
                }
            }
        }
        false
    }
}

impl Default for CurrencySystem {
    fn default() -> Self {
        Self::new()
    }
}
