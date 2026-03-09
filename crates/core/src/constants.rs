/// 最低胡牌胡息
pub const MIN_HUXI_TO_WIN: u8 = 10;
/// 庄家手牌张数
pub const DEALER_CARD_COUNT: usize = 21;
/// 闲家手牌张数
pub const PLAYER_CARD_COUNT: usize = 20;
/// 总牌张数 (小写10种×4张 + 大写10种×4张 = 80张)
pub const TOTAL_CARDS: usize = 80;
/// 最大玩家数
pub const MAX_PLAYERS: usize = 4;
/// 最少玩家数
pub const MIN_PLAYERS: usize = 2;

/// 底分 (基础下注)
pub const BASE_BET: u64 = 1000;
/// 系统抽水比例 (5%)
pub const DEFAULT_RAKE_PERCENTAGE: f64 = 0.05;
/// 每日免费赠送欢乐豆
pub const DAILY_FREE_BEANS: u64 = 5000;
/// 破产补助阈值 — 低于此值可领取补助
pub const BANKRUPTCY_THRESHOLD: u64 = 500;
/// 破产补助金额
pub const BANKRUPTCY_AID_AMOUNT: u64 = 3000;
/// 每日最大补助次数
pub const MAX_DAILY_AID_CLAIMS: u8 = 3;
/// 新玩家初始欢乐豆
pub const INITIAL_BEANS: u64 = 10000;

/// 胡息→舵数 映射表
/// 10~12胡息 = 1舵, 13~15 = 2舵, 16~18 = 3舵 ...
pub const HUXI_TO_DUO: [(u8, u8); 10] = [
    (10, 1),
    (13, 2),
    (16, 3),
    (19, 4),
    (22, 5),
    (25, 6),
    (28, 7),
    (31, 8),
    (34, 9),
    (37, 10),
];

/// 根据胡息计算舵数
pub fn calculate_duo(huxi: u8) -> u8 {
    for (threshold, duo) in HUXI_TO_DUO.iter().rev() {
        if huxi >= *threshold {
            return *duo;
        }
    }
    0
}

/// 基础番数计算 (舵 + 自摸/天胡/地胡 加成)
pub fn calculate_fan(duo: u8, is_zimo: bool, is_tianhu: bool, is_dihu: bool) -> u8 {
    let mut fan = duo;
    if is_zimo {
        fan += 1;
    }
    if is_tianhu {
        fan += 2;
    }
    if is_dihu {
        fan += 2;
    }
    fan
}

/// 根据番数和底分计算最终得分
pub fn calculate_score(fan: u8, base_bet: u64) -> u64 {
    base_bet * fan as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_duo() {
        assert_eq!(calculate_duo(9), 0);
        assert_eq!(calculate_duo(10), 1);
        assert_eq!(calculate_duo(12), 1);
        assert_eq!(calculate_duo(13), 2);
        assert_eq!(calculate_duo(15), 2);
        assert_eq!(calculate_duo(16), 3);
        assert_eq!(calculate_duo(37), 10);
        assert_eq!(calculate_duo(50), 10);
    }

    #[test]
    fn test_calculate_fan() {
        // 普通胡: 1舵 = 1番
        assert_eq!(calculate_fan(1, false, false, false), 1);
        // 自摸: 1舵 + 1 = 2番
        assert_eq!(calculate_fan(1, true, false, false), 2);
        // 天胡: 1舵 + 2 = 3番
        assert_eq!(calculate_fan(1, false, true, false), 3);
        // 地胡: 1舵 + 2 = 3番
        assert_eq!(calculate_fan(1, false, false, true), 3);
        // 自摸天胡: 1舵 + 1 + 2 = 4番
        assert_eq!(calculate_fan(1, true, true, false), 4);
    }

    #[test]
    fn test_calculate_score() {
        assert_eq!(calculate_score(1, 1000), 1000);
        assert_eq!(calculate_score(3, 1000), 3000);
        assert_eq!(calculate_score(10, 100), 1000);
    }

    #[test]
    fn test_constants_valid() {
        assert!(MIN_PLAYERS >= 2);
        assert!(MAX_PLAYERS >= MIN_PLAYERS);
        assert!(DEALER_CARD_COUNT > PLAYER_CARD_COUNT);
        assert!(TOTAL_CARDS == 80);
        assert!(BANKRUPTCY_THRESHOLD < INITIAL_BEANS);
        assert!(DEFAULT_RAKE_PERCENTAGE > 0.0 && DEFAULT_RAKE_PERCENTAGE < 1.0);
    }
}
