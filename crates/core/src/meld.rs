use crate::card::{Card, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeldType {
    Chi,
    Peng,
    SanDa,
    Sao,
    SaoChuan,
    KaiDuo,
    Kan,
}

impl MeldType {
    pub fn base_huxi(&self, is_big: bool) -> u8 {
        match self {
            MeldType::Chi => {
                if is_big { 6 } else { 3 }
            }
            MeldType::Peng => {
                if is_big { 3 } else { 1 }
            }
            MeldType::SanDa => {
                if is_big { 5 } else { 4 }
            }
            MeldType::Sao | MeldType::Kan => {
                if is_big { 6 } else { 3 }
            }
            MeldType::SaoChuan | MeldType::KaiDuo => {
                if is_big { 12 } else { 9 }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meld {
    pub meld_type: MeldType,
    pub cards: Vec<Card>,
    pub from_opponent: bool,
}

impl Meld {
    pub fn new(meld_type: MeldType, cards: Vec<Card>, from_opponent: bool) -> Self {
        Self {
            meld_type,
            cards,
            from_opponent,
        }
    }

    pub fn huxi(&self) -> u8 {
        if self.meld_type == MeldType::Chi {
            // 只有 123 和 2710 有胡息
            if !Self::is_valid_123(&self.cards) && !Self::is_valid_2710(&self.cards) {
                return 0;
            }
        }

        let big_count = self.cards.iter().filter(|c| c.suit == Suit::Big).count();
        let is_big = if self.meld_type == MeldType::SanDa {
            big_count >= 2
        } else {
            self.cards.iter().any(|c| c.suit == Suit::Big)
        };
        self.meld_type.base_huxi(is_big)
    }

    pub fn is_valid_chi(cards: &[Card]) -> bool {
        Self::is_valid_123(cards) || Self::is_valid_2710(cards)
    }

    pub fn is_valid_123(cards: &[Card]) -> bool {
        if cards.len() != 3 {
            return false;
        }

        let first_suit = cards[0].suit;
        if cards.iter().any(|c| c.suit != first_suit) {
            return false;
        }

        let mut values: Vec<u8> = cards.iter().map(|c| c.value.as_u8()).collect();
        values.sort();

        values[0] == 1 && values[1] == 2 && values[2] == 3
    }

    pub fn is_valid_2710(cards: &[Card]) -> bool {
        if cards.len() != 3 {
            return false;
        }

        let first_suit = cards[0].suit;
        if cards.iter().any(|c| c.suit != first_suit) {
            return false;
        }

        let mut values: Vec<u8> = cards.iter().map(|c| c.value.as_u8()).collect();
        values.sort();

        values == vec![2, 7, 10]
    }

    pub fn is_valid_san_da(cards: &[Card]) -> bool {
        if cards.len() != 3 {
            return false;
        }

        let big_count = cards.iter().filter(|c| c.suit == Suit::Big).count();
        let small_count = cards.iter().filter(|c| c.suit == Suit::Small).count();

        (big_count == 2 && small_count == 1) || (big_count == 1 && small_count == 2)
    }

    pub fn is_valid_peng(cards: &[Card]) -> bool {
        if cards.len() != 3 {
            return false;
        }
        cards
            .iter()
            .all(|c| c.suit == cards[0].suit && c.value == cards[0].value)
    }

    pub fn is_valid_kan(cards: &[Card]) -> bool {
        if cards.len() != 3 {
            return false;
        }
        cards
            .iter()
            .all(|c| c.suit == cards[0].suit && c.value == cards[0].value)
    }

    pub fn is_valid_sao(self_cards: &[Card], new_card: &Card) -> bool {
        if self_cards.len() != 2 {
            return false;
        }
        self_cards.iter().all(|c| c == new_card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CardValue;

    #[test]
    fn test_valid_chi() {
        let chi = vec![
            Card::new(Suit::Small, CardValue::One),
            Card::new(Suit::Small, CardValue::Two),
            Card::new(Suit::Small, CardValue::Three),
        ];
        assert!(Meld::is_valid_chi(&chi));
    }

    #[test]
    fn test_invalid_chi_different_suits() {
        let chi = vec![
            Card::new(Suit::Small, CardValue::One),
            Card::new(Suit::Big, CardValue::Two),
            Card::new(Suit::Small, CardValue::Three),
        ];
        assert!(!Meld::is_valid_chi(&chi));
    }

    #[test]
    fn test_valid_2710() {
        let erqishi = vec![
            Card::new(Suit::Small, CardValue::Two),
            Card::new(Suit::Small, CardValue::Seven),
            Card::new(Suit::Small, CardValue::Ten),
        ];
        assert!(Meld::is_valid_2710(&erqishi));
    }

    #[test]
    fn test_valid_peng() {
        let peng = vec![
            Card::new(Suit::Small, CardValue::Five),
            Card::new(Suit::Small, CardValue::Five),
            Card::new(Suit::Small, CardValue::Five),
        ];
        assert!(Meld::is_valid_peng(&peng));
    }
}
