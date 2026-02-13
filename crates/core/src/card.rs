use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Suit {
    Small,
    Big,
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Suit::Small => write!(f, "小"),
            Suit::Big => write!(f, "大"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardValue {
    One = 1,
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
}

impl CardValue {
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    pub fn is_red(&self) -> bool {
        matches!(self, CardValue::Two | CardValue::Seven | CardValue::Ten)
    }
}

impl fmt::Display for CardValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            CardValue::One => "一",
            CardValue::Two => "二",
            CardValue::Three => "三",
            CardValue::Four => "四",
            CardValue::Five => "五",
            CardValue::Six => "六",
            CardValue::Seven => "七",
            CardValue::Eight => "八",
            CardValue::Nine => "九",
            CardValue::Ten => "十",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub value: CardValue,
}

impl Card {
    pub fn new(suit: Suit, value: CardValue) -> Self {
        Self { suit, value }
    }

    pub fn is_red(&self) -> bool {
        self.value.is_red()
    }

    pub fn card_type(&self) -> CardType {
        if self.is_red() {
            CardType::Red
        } else {
            CardType::Black
        }
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suit_str = match self.suit {
            Suit::Small => "",
            Suit::Big => "",
        };
        let value_str = match self.value {
            CardValue::One => if self.suit == Suit::Big { "壹" } else { "一" },
            CardValue::Two => if self.suit == Suit::Big { "贰" } else { "二" },
            CardValue::Three => if self.suit == Suit::Big { "叁" } else { "三" },
            CardValue::Four => if self.suit == Suit::Big { "肆" } else { "四" },
            CardValue::Five => if self.suit == Suit::Big { "伍" } else { "五" },
            CardValue::Six => if self.suit == Suit::Big { "陆" } else { "六" },
            CardValue::Seven => if self.suit == Suit::Big { "柒" } else { "七" },
            CardValue::Eight => if self.suit == Suit::Big { "捌" } else { "八" },
            CardValue::Nine => if self.suit == Suit::Big { "玖" } else { "九" },
            CardValue::Ten => if self.suit == Suit::Big { "拾" } else { "十" },
        };
        write!(f, "{}{}", suit_str, value_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Red,
    Black,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_creation() {
        let card = Card::new(Suit::Small, CardValue::One);
        assert_eq!(card.suit, Suit::Small);
        assert_eq!(card.value, CardValue::One);
        assert!(!card.is_red());
    }

    #[test]
    fn test_red_cards() {
        let red_small = Card::new(Suit::Small, CardValue::Two);
        let red_big = Card::new(Suit::Big, CardValue::Seven);
        let red_ten = Card::new(Suit::Small, CardValue::Ten);

        assert!(red_small.is_red());
        assert!(red_big.is_red());
        assert!(red_ten.is_red());
    }

    #[test]
    fn test_card_display() {
        let card1 = Card::new(Suit::Small, CardValue::One);
        let card2 = Card::new(Suit::Big, CardValue::One);
        assert_eq!(card1.to_string(), "一");
        assert_eq!(card2.to_string(), "壹");
    }
}
