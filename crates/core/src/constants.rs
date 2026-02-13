pub const MIN_HUXI_TO_WIN: u8 = 10;
pub const DEALER_CARD_COUNT: usize = 21;
pub const PLAYER_CARD_COUNT: usize = 20;
pub const TOTAL_CARDS: usize = 80;
pub const MAX_PLAYERS: usize = 4;
pub const MIN_PLAYERS: usize = 2;

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

pub fn calculate_duo(huxi: u8) -> u8 {
    for (threshold, duo) in HUXI_TO_DUO.iter().rev() {
        if huxi >= *threshold {
            return *duo;
        }
    }
    0
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
    }
}
