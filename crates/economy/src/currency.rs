use guilin_paizi_core::PlayerId;
use guilin_paizi_core::constants::{BANKRUPTCY_THRESHOLD, BANKRUPTCY_AID_AMOUNT, MAX_DAILY_AID_CLAIMS, DAILY_FREE_BEANS};
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
    BankruptcyAid,
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

    pub fn add(&mut self, player_id: PlayerId, amount: u64, transaction_type: TransactionType, description: impl Into<String>) {
        self.balance += amount;
        self.total_earned += amount;
        self.transaction_history.push(Transaction {
            player_id,
            amount: amount as i64,
            transaction_type,
            timestamp: Utc::now(),
            description: description.into(),
        });
    }

    pub fn deduct(&mut self, player_id: PlayerId, amount: u64, transaction_type: TransactionType, description: impl Into<String>) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            self.total_spent += amount;
            self.transaction_history.push(Transaction {
                player_id,
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

    /// 是否达到破产补助条件
    pub fn is_bankrupt(&self) -> bool {
        self.balance < BANKRUPTCY_THRESHOLD
    }

    /// 最近N条流水
    pub fn recent_transactions(&self, n: usize) -> &[Transaction] {
        let start = self.transaction_history.len().saturating_sub(n);
        &self.transaction_history[start..]
    }
}

pub struct CurrencySystem {
    player_balances: HashMap<PlayerId, HappyBeans>,
    daily_claims: HashMap<PlayerId, DateTime<Utc>>,
    daily_aid_claims: HashMap<PlayerId, (DateTime<Utc>, u8)>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        Self {
            player_balances: HashMap::new(),
            daily_claims: HashMap::new(),
            daily_aid_claims: HashMap::new(),
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

    pub fn claim_daily_bonus(&mut self, player_id: PlayerId) -> Option<u64> {
        if self.can_claim_daily(player_id) {
            if let Some(beans) = self.player_balances.get_mut(&player_id) {
                beans.add(player_id, DAILY_FREE_BEANS, TransactionType::DailyBonus, "每日签到奖励");
                self.daily_claims.insert(player_id, Utc::now());
                return Some(beans.balance);
            }
        }
        None
    }

    /// 破产补助 —— 余额低于阈值可每日最多领取 MAX_DAILY_AID_CLAIMS 次
    pub fn claim_bankruptcy_aid(&mut self, player_id: PlayerId) -> Result<u64, String> {
        let beans = self.player_balances.get(&player_id)
            .ok_or_else(|| "玩家不存在".to_string())?;

        if !beans.is_bankrupt() {
            return Err(format!("余额 {} 未达到破产补助条件（需低于 {}）", beans.balance, BANKRUPTCY_THRESHOLD));
        }

        // 检查今日领取次数
        let now = Utc::now();
        if let Some((last_date, count)) = self.daily_aid_claims.get(&player_id) {
            let days_since = now.signed_duration_since(*last_date).num_days();
            if days_since == 0 && *count >= MAX_DAILY_AID_CLAIMS {
                return Err(format!("今日补助次数已用完（最多 {} 次）", MAX_DAILY_AID_CLAIMS));
            }
        }

        // 发放补助
        let beans = self.player_balances.get_mut(&player_id).unwrap();
        beans.add(player_id, BANKRUPTCY_AID_AMOUNT, TransactionType::BankruptcyAid, "破产补助");

        // 更新领取记录
        let entry = self.daily_aid_claims.entry(player_id).or_insert((now, 0));
        let days_since = now.signed_duration_since(entry.0).num_days();
        if days_since >= 1 {
            *entry = (now, 1);
        } else {
            entry.1 += 1;
        }

        Ok(beans.balance)
    }

    pub fn transfer(&mut self, from: PlayerId, to: PlayerId, amount: u64, description: impl Into<String>) -> bool {
        let desc: String = description.into();
        if let Some(from_beans) = self.player_balances.get_mut(&from) {
            if from_beans.deduct(from, amount, TransactionType::Loss, &desc) {
                if let Some(to_beans) = self.player_balances.get_mut(&to) {
                    to_beans.add(to, amount, TransactionType::Win, "赢得游戏");
                    return true;
                }
            }
        }
        false
    }

    /// 系统抽水扣除
    pub fn collect_rake(&mut self, player_id: PlayerId, amount: u64) -> bool {
        if let Some(beans) = self.player_balances.get_mut(&player_id) {
            beans.deduct(player_id, amount, TransactionType::Rake, "系统抽水")
        } else {
            false
        }
    }
}

impl Default for CurrencySystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guilin_paizi_core::PlayerId;

    fn setup_system() -> (CurrencySystem, PlayerId) {
        let mut sys = CurrencySystem::new();
        let pid = PlayerId::new();
        sys.register_player(pid, 10000);
        (sys, pid)
    }

    #[test]
    fn test_register_and_balance() {
        let (sys, pid) = setup_system();
        assert_eq!(sys.get_balance(pid), Some(10000));
    }

    #[test]
    fn test_add_and_deduct() {
        let (mut sys, pid) = setup_system();
        let beans = sys.get_beans_mut(pid).unwrap();

        beans.add(pid, 500, TransactionType::Win, "胜利");
        assert_eq!(beans.balance, 10500);

        let ok = beans.deduct(pid, 300, TransactionType::Loss, "失败");
        assert!(ok);
        assert_eq!(beans.balance, 10200);

        let ok = beans.deduct(pid, 99999, TransactionType::Loss, "超额");
        assert!(!ok);
        assert_eq!(beans.balance, 10200);
    }

    #[test]
    fn test_daily_bonus() {
        let (mut sys, pid) = setup_system();
        assert!(sys.can_claim_daily(pid));

        let balance = sys.claim_daily_bonus(pid);
        assert!(balance.is_some());
        assert_eq!(balance.unwrap(), 10000 + DAILY_FREE_BEANS);

        // 同一天再次领取应返回 None
        assert!(!sys.can_claim_daily(pid));
        assert!(sys.claim_daily_bonus(pid).is_none());
    }

    #[test]
    fn test_bankruptcy_aid() {
        let mut sys = CurrencySystem::new();
        let pid = PlayerId::new();
        sys.register_player(pid, 100); // 低于阈值

        // 应该可以领取补助
        let result = sys.claim_bankruptcy_aid(pid);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100 + BANKRUPTCY_AID_AMOUNT);
    }

    #[test]
    fn test_bankruptcy_aid_not_qualified() {
        let (mut sys, pid) = setup_system(); // 10000余额 > 阈值
        let result = sys.claim_bankruptcy_aid(pid);
        assert!(result.is_err());
    }

    #[test]
    fn test_transfer() {
        let mut sys = CurrencySystem::new();
        let p1 = PlayerId::new();
        let p2 = PlayerId::new();
        sys.register_player(p1, 5000);
        sys.register_player(p2, 5000);

        assert!(sys.transfer(p1, p2, 2000, "对局结算"));
        assert_eq!(sys.get_balance(p1), Some(3000));
        assert_eq!(sys.get_balance(p2), Some(7000));
    }

    #[test]
    fn test_transfer_insufficient() {
        let mut sys = CurrencySystem::new();
        let p1 = PlayerId::new();
        let p2 = PlayerId::new();
        sys.register_player(p1, 100);
        sys.register_player(p2, 100);

        assert!(!sys.transfer(p1, p2, 500, "超额转账"));
        assert_eq!(sys.get_balance(p1), Some(100));
        assert_eq!(sys.get_balance(p2), Some(100));
    }

    #[test]
    fn test_rake_collection() {
        let (mut sys, pid) = setup_system();
        assert!(sys.collect_rake(pid, 500));
        assert_eq!(sys.get_balance(pid), Some(9500));
    }

    #[test]
    fn test_transaction_history() {
        let (mut sys, pid) = setup_system();
        let beans = sys.get_beans_mut(pid).unwrap();
        beans.add(pid, 100, TransactionType::Win, "赢");
        beans.deduct(pid, 50, TransactionType::Loss, "输");

        let recent = beans.recent_transactions(5);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].amount, 100);
        assert_eq!(recent[1].amount, -50);
    }

    #[test]
    fn test_is_bankrupt() {
        let mut beans = HappyBeans::new(100);
        assert!(beans.is_bankrupt());

        beans.balance = BANKRUPTCY_THRESHOLD;
        assert!(!beans.is_bankrupt());

        beans.balance = BANKRUPTCY_THRESHOLD - 1;
        assert!(beans.is_bankrupt());
    }
}
