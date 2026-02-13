use crate::card::Card;
use crate::meld::{Meld, MeldType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hand {
    cards: Vec<Card>,
    melds: Vec<Meld>,
}

impl Hand {
    pub fn new(cards: Vec<Card>) -> Self {
        Self {
            cards,
            melds: Vec::new(),
        }
    }

    pub fn add_card(&mut self, card: Card) {
        self.cards.push(card);
        self.sort();
    }

    pub fn remove_card(&mut self, index: usize) -> Option<Card> {
        if index < self.cards.len() {
            Some(self.cards.remove(index))
        } else {
            None
        }
    }

    pub fn find_card(&self, card: &Card) -> Option<usize> {
        self.cards.iter().position(|c| c == card)
    }

    pub fn cards(&self) -> &[Card] {
        &self.cards
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn sort(&mut self) {
        self.cards.sort_by(|a, b| {
            let suit_order = (a.suit as u8).cmp(&(b.suit as u8));
            if suit_order == std::cmp::Ordering::Equal {
                (a.value as u8).cmp(&(b.value as u8))
            } else {
                suit_order
            }
        });
    }

    pub fn add_meld(&mut self, meld: Meld) {
        self.melds.push(meld);
    }

    pub fn melds(&self) -> &[Meld] {
        &self.melds
    }

    pub fn get_card_counts(&self) -> HashMap<Card, u8> {
        let mut counts = HashMap::new();
        for card in &self.cards {
            *counts.entry(*card).or_insert(0) += 1;
        }
        counts
    }

    pub fn has_meld(&self, cards: &[Card]) -> bool {
        let counts = self.get_card_counts();
        for card in cards {
            if counts.get(card).copied().unwrap_or(0) == 0 {
                return false;
            }
        }
        true
    }

    pub fn can_peng(&self, card: &Card) -> bool {
        let counts = self.get_card_counts();
        counts.get(card).copied().unwrap_or(0) >= 2
    }

    pub fn can_sao(&self, card: &Card) -> bool {
        let counts = self.get_card_counts();
        counts.get(card).copied().unwrap_or(0) >= 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardValue, Suit};

    #[test]
    fn test_hand_operations() {
        let cards = vec![
            Card::new(Suit::Small, CardValue::One),
            Card::new(Suit::Small, CardValue::Two),
        ];
        let mut hand = Hand::new(cards);
        
        hand.add_card(Card::new(Suit::Small, CardValue::Three));
        assert_eq!(hand.len(), 3);
        
        let removed = hand.remove_card(0);
        assert!(removed.is_some());
        assert_eq!(hand.len(), 2);
    }

    #[test]
    fn test_peng_detection() {
        let cards = vec![
            Card::new(Suit::Small, CardValue::One),
            Card::new(Suit::Small, CardValue::One),
            Card::new(Suit::Small, CardValue::Two),
        ];
        let hand = Hand::new(cards);
        
        let peng_card = Card::new(Suit::Small, CardValue::One);
        assert!(hand.can_peng(&peng_card));
        
        let no_peng_card = Card::new(Suit::Small, CardValue::Two);
        assert!(!hand.can_peng(&no_peng_card));
    }
}
