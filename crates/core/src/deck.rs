use crate::card::{Card, CardValue, Suit};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(80);

        for suit in [Suit::Small, Suit::Big] {
            for value in [
                CardValue::One,
                CardValue::Two,
                CardValue::Three,
                CardValue::Four,
                CardValue::Five,
                CardValue::Six,
                CardValue::Seven,
                CardValue::Eight,
                CardValue::Nine,
                CardValue::Ten,
            ] {
                for _ in 0..4 {
                    cards.push(Card::new(suit, value));
                }
            }
        }

        Self { cards }
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        let mut drawn = Vec::with_capacity(n);
        for _ in 0..n {
            if let Some(card) = self.draw() {
                drawn.push(card);
            } else {
                break;
            }
        }
        drawn
    }

    pub fn remaining(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn peek_remaining(&self) -> &[Card] {
        &self.cards
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_size() {
        let deck = Deck::new();
        assert_eq!(deck.remaining(), 80);
    }

    #[test]
    fn test_draw() {
        let mut deck = Deck::new();
        let card = deck.draw();
        assert!(card.is_some());
        assert_eq!(deck.remaining(), 79);
    }

    #[test]
    fn test_draw_n() {
        let mut deck = Deck::new();
        let cards = deck.draw_n(5);
        assert_eq!(cards.len(), 5);
        assert_eq!(deck.remaining(), 75);
    }
}
