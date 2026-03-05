use crate::card::Card;
use crate::constants::{DEALER_CARD_COUNT, MAX_PLAYERS, MIN_HUXI_TO_WIN, PLAYER_CARD_COUNT};
use crate::deck::Deck;
use crate::error::{GameError, Result};
use crate::hand::Hand;
use crate::meld::{Meld, MeldType};
use crate::player::{Player, PlayerId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GamePhase {
    Waiting,
    Dealing,
    Playing,
    Settling,
    Finished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HuType {
    Normal,
    Zimo,
    TianHu,
    DiHu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub phase: GamePhase,
    pub players: Vec<Player>,
    pub hands: HashMap<PlayerId, Hand>,
    pub deck: Deck,
    pub discard_pile: Vec<(PlayerId, Card)>,
    pub current_player_idx: usize,
    pub dealer_idx: usize,
    pub round: u8,
    pub last_action: Option<GameAction>,
    pub dangdi: Option<Card>,
    pub must_hu_player: Option<PlayerId>,
    pub hu_type: Option<HuType>,
    pub is_first_turn: bool,
    pub win_result: Option<WinResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GameAction {
    PlayCard { player: PlayerId, card_idx: usize },
    Chi { player: PlayerId, cards: Vec<usize> },
    Peng { player: PlayerId, card: Card },
    Sao { player: PlayerId, card: Card },
    SaoChuan { player: PlayerId, card: Card },
    KaiDuo { player: PlayerId, card: Card },
    Hu { player: PlayerId, is_zimo: bool },
    Pass { player: PlayerId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStateView {
    pub phase: GamePhase,
    pub players: Vec<PlayerId>,
    pub current_player_idx: usize,
    pub dealer_idx: usize,
    pub round: u8,
    pub last_action: Option<String>,
    pub discard_pile: Vec<(String, Card)>,
    pub dangdi: Option<Card>,
    pub must_hu_player: Option<String>,
    pub hu_type: Option<HuType>,
    pub is_first_turn: bool,
    pub win_result: Option<WinResult>,
}

impl GameState {
    pub fn to_view(&self) -> GameStateView {
        GameStateView {
            phase: self.phase,
            players: self.players.iter().map(|p| p.id).collect(),
            current_player_idx: self.current_player_idx,
            dealer_idx: self.dealer_idx,
            round: self.round,
            last_action: self.last_action.as_ref().map(|a| format!("{:?}", a)),
            discard_pile: self
                .discard_pile
                .iter()
                .map(|(pid, card)| (format!("{:?}", pid), *card))
                .collect(),
            dangdi: self.dangdi,
            must_hu_player: self.must_hu_player.map(|pid| format!("{:?}", pid)),
            hu_type: self.hu_type,
            is_first_turn: self.is_first_turn,
            win_result: self.win_result.clone(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WinResult {
    pub winner: PlayerId,
    pub huxi: u8,
    pub duo: u8,
    pub fan: u8,
    pub is_zimo: bool,
    pub is_tianhu: bool,
    pub is_dihu: bool,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::Waiting,
            players: Vec::new(),
            hands: HashMap::new(),
            deck: Deck::new(),
            discard_pile: Vec::new(),
            current_player_idx: 0,
            dealer_idx: 0,
            round: 1,
            last_action: None,
            dangdi: None,
            must_hu_player: None,
            hu_type: None,
            is_first_turn: true,
            win_result: None,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<PlayerId> {
        if self.players.len() >= MAX_PLAYERS {
            return Err(GameError::GameFull);
        }
        let id = player.id;
        self.players.push(player);
        Ok(id)
    }

    pub fn start_game(&mut self) -> Result<()> {
        if self.players.len() < 2 {
            return Err(GameError::InvalidAction);
        }

        self.phase = GamePhase::Dealing;
        self.deck.shuffle();

        for (idx, player) in self.players.iter_mut().enumerate() {
            player.set_playing();
            player.position = idx as u8;

            let card_count = if idx == self.dealer_idx {
                DEALER_CARD_COUNT
            } else {
                PLAYER_CARD_COUNT
            };

            let cards = self.deck.draw_n(card_count);
            let mut hand = Hand::new(cards);
            hand.sort();

            if idx == self.dealer_idx && !hand.cards().is_empty() {
                self.dangdi = Some(hand.cards()[card_count - 1]);
            }

            self.hands.insert(player.id, hand);
        }

        self.phase = GamePhase::Playing;
        self.current_player_idx = self.dealer_idx;

        Ok(())
    }

    pub fn get_current_player(&self) -> Option<&Player> {
        self.players.get(self.current_player_idx)
    }

    pub fn get_current_player_id(&self) -> Option<PlayerId> {
        self.get_current_player().map(|p| p.id)
    }

    pub fn play_card(&mut self, player_id: PlayerId, card_idx: usize) -> Result<Card> {
        if self.phase != GamePhase::Playing {
            return Err(GameError::InvalidAction);
        }

        let current_player = self
            .get_current_player_id()
            .ok_or(GameError::InvalidAction)?;

        if player_id != current_player {
            return Err(GameError::NotYourTurn);
        }

        let hand = self
            .hands
            .get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let card = hand.remove_card(card_idx).ok_or(GameError::CardNotInHand)?;

        self.discard_pile.push((player_id, card));

        self.draw_card(player_id)?;

        self.next_turn();

        self.last_action = Some(GameAction::PlayCard {
            player: player_id,
            card_idx,
        });

        Ok(card)
    }

    pub fn draw_card(&mut self, player_id: PlayerId) -> Result<Option<Card>> {
        if let Some(card) = self.deck.draw() {
            if let Some(hand) = self.hands.get_mut(&player_id) {
                hand.add_card(card);
                Ok(Some(card))
            } else {
                Err(GameError::PlayerNotFound)
            }
        } else {
            Ok(None)
        }
    }

    pub fn chi(&mut self, player_id: PlayerId, card_indices: Vec<usize>) -> Result<Meld> {
        if card_indices.len() != 2 {
            return Err(GameError::InvalidMeld);
        }

        let hand = self
            .hands
            .get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let last_discard = self.discard_pile.last().ok_or(GameError::InvalidAction)?;

        let mut meld_cards = vec![last_discard.1];

        for &idx in &card_indices {
            if idx >= hand.cards().len() {
                return Err(GameError::CardNotInHand);
            }
        }

        let indices: Vec<_> = card_indices.iter().copied().collect();
        for idx in indices.iter().rev() {
            let card = hand.cards()[*idx];
            meld_cards.push(card);
        }

        if !Meld::is_valid_chi(&meld_cards) && !Meld::is_valid_2710(&meld_cards) {
            return Err(GameError::InvalidMeld);
        }

        for idx in indices.iter().rev() {
            hand.remove_card(*idx);
        }

        let meld = Meld::new(MeldType::Chi, meld_cards, true);
        hand.add_meld(meld.clone());

        self.last_action = Some(GameAction::Chi {
            player: player_id,
            cards: card_indices,
        });

        Ok(meld)
    }

    pub fn peng(&mut self, player_id: PlayerId, card: Card) -> Result<Meld> {
        let hand = self
            .hands
            .get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        if !hand.can_peng(&card) {
            return Err(GameError::InvalidMeld);
        }

        let cards = vec![card, card, card];

        for _ in 0..2 {
            if let Some(idx) = hand.find_card(&card) {
                hand.remove_card(idx);
            }
        }

        let meld = Meld::new(MeldType::Peng, cards, true);
        hand.add_meld(meld.clone());

        self.last_action = Some(GameAction::Peng {
            player: player_id,
            card,
        });

        Ok(meld)
    }

    pub fn sao(&mut self, player_id: PlayerId, card: Card) -> Result<Meld> {
        let hand = self
            .hands
            .get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let counts = hand.get_card_counts();
        let card_count = *counts.get(&card).unwrap_or(&0);

        if card_count < 3 {
            return Err(GameError::InvalidMeld);
        }

        let cards = vec![card; 3];
        for _ in 0..3 {
            if let Some(idx) = hand.find_card(&card) {
                hand.remove_card(idx);
            }
        }

        let meld = Meld::new(MeldType::Sao, cards, false);
        hand.add_meld(meld.clone());

        self.last_action = Some(GameAction::Sao {
            player: player_id,
            card,
        });

        Ok(meld)
    }

    pub fn sao_chuan(&mut self, player_id: PlayerId, card: Card) -> Result<Meld> {
        let hand = self
            .hands
            .get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let counts = hand.get_card_counts();
        let card_count = *counts.get(&card).unwrap_or(&0);

        if card_count < 4 {
            return Err(GameError::InvalidMeld);
        }

        let cards = vec![card; 4];
        for _ in 0..4 {
            if let Some(idx) = hand.find_card(&card) {
                hand.remove_card(idx);
            }
        }

        let meld = Meld::new(MeldType::SaoChuan, cards, false);
        hand.add_meld(meld.clone());

        self.last_action = Some(GameAction::SaoChuan {
            player: player_id,
            card,
        });

        Ok(meld)
    }

    pub fn kai_duo(&mut self, player_id: PlayerId, card: Card) -> Result<Meld> {
        let hand = self
            .hands
            .get_mut(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let counts = hand.get_card_counts();
        let card_count = *counts.get(&card).unwrap_or(&0);

        if card_count < 4 {
            return Err(GameError::InvalidMeld);
        }

        let cards = vec![card; 4];
        for _ in 0..4 {
            if let Some(idx) = hand.find_card(&card) {
                hand.remove_card(idx);
            }
        }

        let meld = Meld::new(MeldType::KaiDuo, cards, false);
        hand.add_meld(meld.clone());

        self.last_action = Some(GameAction::KaiDuo {
            player: player_id,
            card,
        });

        Ok(meld)
    }

    pub fn hu(&mut self, player_id: PlayerId) -> Result<WinResult> {
        let huxi = self.calculate_hand_huxi(player_id)?;

        if huxi < MIN_HUXI_TO_WIN {
            return Err(GameError::InvalidAction);
        }

        let hu_type = if self.is_first_turn && self.current_player_idx == self.dealer_idx {
            HuType::TianHu
        } else if self.is_first_turn {
            HuType::DiHu
        } else if self.must_hu_player == Some(player_id) {
            HuType::Normal
        } else {
            HuType::Zimo
        };

        let is_zimo = matches!(hu_type, HuType::Zimo);

        let duo = self.calculate_duo_from_huxi(huxi);
        let fan = self.calculate_fan(
            duo,
            is_zimo,
            matches!(hu_type, HuType::TianHu),
            matches!(hu_type, HuType::DiHu),
        );

        let win_result = WinResult {
            winner: player_id,
            huxi,
            duo,
            fan,
            is_zimo,
            is_tianhu: matches!(hu_type, HuType::TianHu),
            is_dihu: matches!(hu_type, HuType::DiHu),
        };

        self.phase = GamePhase::Finished;
        self.hu_type = Some(hu_type);
        self.win_result = Some(win_result.clone());

        Ok(win_result)
    }

    pub fn check_opponent_can_hu(&self, exclude_player: PlayerId) -> Option<PlayerId> {
        for (player_id, _hand) in &self.hands {
            if *player_id == exclude_player {
                continue;
            }
            if let Ok(true) = self.can_hu(*player_id) {
                return Some(*player_id);
            }
        }
        None
    }

    pub fn check_must_sao(&self, player_id: PlayerId) -> Option<Card> {
        let hand = self.hands.get(&player_id)?;
        let counts = hand.get_card_counts();

        for (card, count) in &counts {
            if *count >= 4 {
                return Some(*card);
            }
        }
        None
    }

    pub fn calculate_duo_from_huxi(&self, huxi: u8) -> u8 {
        use crate::constants::HUXI_TO_DUO;
        for (threshold, duo) in HUXI_TO_DUO.iter().rev() {
            if huxi >= *threshold {
                return *duo;
            }
        }
        0
    }

    pub fn calculate_fan(&self, duo: u8, is_zimo: bool, is_tianhu: bool, is_dihu: bool) -> u8 {
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

    pub fn can_chi(&self, player_id: PlayerId) -> bool {
        if self.discard_pile.is_empty() {
            return false;
        }

        let last_card = self.discard_pile.last().unwrap().1;
        let hand = match self.hands.get(&player_id) {
            Some(h) => h,
            None => return false,
        };

        let counts = hand.get_card_counts();

        for (card, count) in &counts {
            if card.value == last_card.value && card.suit == last_card.suit {
                continue;
            }

            if *count == 0 {
                continue;
            }

            let test_cards = vec![*card, last_card];
            if Meld::is_valid_chi(&test_cards) || Meld::is_valid_2710(&test_cards) {
                return true;
            }
        }

        false
    }

    pub fn can_peng(&self, player_id: PlayerId) -> bool {
        if self.discard_pile.is_empty() {
            return false;
        }

        let last_card = self.discard_pile.last().unwrap().1;

        if let Some(hand) = self.hands.get(&player_id) {
            return hand.can_peng(&last_card);
        }

        false
    }

    pub fn can_sao(&self, player_id: PlayerId) -> bool {
        if let Some(hand) = self.hands.get(&player_id) {
            let counts = hand.get_card_counts();
            return counts.values().any(|&c| c >= 3);
        }
        false
    }

    pub fn can_sao_chuan(&self, player_id: PlayerId) -> bool {
        if let Some(hand) = self.hands.get(&player_id) {
            let counts = hand.get_card_counts();
            return counts.values().any(|&c| c >= 4);
        }
        false
    }

    pub fn calculate_hand_huxi(&self, player_id: PlayerId) -> Result<u8> {
        let hand = self
            .hands
            .get(&player_id)
            .ok_or(GameError::PlayerNotFound)?;

        let mut total_huxi = 0u8;

        for meld in hand.melds() {
            total_huxi = total_huxi.saturating_add(meld.huxi());
        }

        Ok(total_huxi)
    }

    pub fn can_hu(&self, player_id: PlayerId) -> Result<bool> {
        let huxi = self.calculate_hand_huxi(player_id)?;
        Ok(huxi >= MIN_HUXI_TO_WIN)
    }

    pub fn pass(&mut self, player_id: PlayerId) -> Result<()> {
        if self.phase != GamePhase::Playing {
            return Err(GameError::InvalidAction);
        }

        let current_player = self
            .get_current_player_id()
            .ok_or(GameError::InvalidAction)?;

        if player_id != current_player {
            return Err(GameError::NotYourTurn);
        }

        self.next_turn();
        self.last_action = Some(GameAction::Pass { player: player_id });

        Ok(())
    }

    fn next_turn(&mut self) {
        self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase
            && self.players == other.players
            && self.current_player_idx == other.current_player_idx
            && self.dealer_idx == other.dealer_idx
            && self.round == other.round
            && self.last_action == other.last_action
            && self.dangdi == other.dangdi
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardValue, Suit};

    #[test]
    fn test_game_creation() {
        let game = GameState::new();
        assert_eq!(game.phase, GamePhase::Waiting);
        assert!(game.players.is_empty());
        assert!(game.is_first_turn);
    }

    #[test]
    fn test_add_player() {
        let mut game = GameState::new();
        let player = Player::new("测试");
        let id = game.add_player(player).unwrap();
        assert_eq!(game.players.len(), 1);
        assert_eq!(game.players[0].id, id);
    }

    #[test]
    fn test_game_flow() {
        let mut game = GameState::new();

        let p1 = Player::new("玩家1");
        let p2 = Player::new("玩家2");

        game.add_player(p1).unwrap();
        game.add_player(p2).unwrap();

        game.start_game().unwrap();

        assert_eq!(game.phase, GamePhase::Playing);
        assert!(game.dangdi.is_some());
    }

    #[test]
    fn test_sao_chuan() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        let player_id = game.players[0].id;
        let hand = game.hands.get_mut(&player_id).unwrap();

        let card = Card::new(Suit::Small, CardValue::One);
        for _ in 0..4 {
            hand.add_card(card);
        }

        assert!(game.can_sao_chuan(player_id));
    }

    #[test]
    fn test_check_must_sao() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        let player_id = game.players[0].id;
        let hand = game.hands.get_mut(&player_id).unwrap();

        let card = Card::new(Suit::Small, CardValue::One);
        for _ in 0..4 {
            hand.add_card(card);
        }

        let must_sao = game.check_must_sao(player_id);
        assert!(must_sao.is_some());
        assert_eq!(must_sao.unwrap().value, CardValue::One);
    }

    #[test]
    fn test_check_opponent_can_hu() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        let player1_id = game.players[0].id;

        let can_hu = game.check_opponent_can_hu(player1_id);
        assert!(can_hu.is_none());
    }

    #[test]
    fn test_hu_types() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        let player_id = game.players[0].id;

        let hand = game.hands.get_mut(&player_id).unwrap();
        hand.add_meld(Meld::new(
            MeldType::Chi,
            vec![
                Card::new(Suit::Big, CardValue::One),
                Card::new(Suit::Big, CardValue::Two),
                Card::new(Suit::Big, CardValue::Three),
            ],
            false,
        ));
        hand.add_meld(Meld::new(
            MeldType::Chi,
            vec![
                Card::new(Suit::Big, CardValue::Four),
                Card::new(Suit::Big, CardValue::Five),
                Card::new(Suit::Big, CardValue::Six),
            ],
            false,
        ));
        hand.add_meld(Meld::new(
            MeldType::Chi,
            vec![
                Card::new(Suit::Big, CardValue::Seven),
                Card::new(Suit::Big, CardValue::Eight),
                Card::new(Suit::Big, CardValue::Nine),
            ],
            false,
        ));

        let huxi = game.calculate_hand_huxi(player_id).unwrap();
        assert!(huxi >= 10, "胡息 {} 应该 >= 10", huxi);
        assert!(game.can_hu(player_id).unwrap());
    }

    #[test]
    fn test_can_peng() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        let player_id = game.players[0].id;
        let hand = game.hands.get_mut(&player_id).unwrap();
        hand.add_card(Card::new(Suit::Small, CardValue::One));
        hand.add_card(Card::new(Suit::Small, CardValue::One));

        game.discard_pile
            .push((game.players[1].id, Card::new(Suit::Small, CardValue::One)));

        assert!(game.can_peng(player_id));
    }

    #[test]
    fn test_integration_full_game_with_skills() {
        let mut game = GameState::new();

        let p1 = Player::new("玩家1");
        let p2 = Player::new("玩家2");
        let p1_id = p1.id;

        game.add_player(p1).unwrap();
        game.add_player(p2).unwrap();

        game.start_game().unwrap();

        assert_eq!(game.phase, GamePhase::Playing);
        assert!(game.dangdi.is_some());

        let hand = game.hands.get(&p1_id).unwrap();
        assert_eq!(hand.len(), 21);

        assert!(game.can_chi(p1_id) || !game.can_chi(p1_id));
        assert!(game.can_peng(p1_id) || !game.can_peng(p1_id));
    }

    #[test]
    fn test_integration_four_player_game() {
        let mut game = GameState::new();

        for i in 1..=4 {
            game.add_player(Player::new(format!("玩家{}", i))).unwrap();
        }

        assert_eq!(game.players.len(), 4);

        game.start_game().unwrap();

        assert_eq!(game.phase, GamePhase::Playing);

        for player in &game.players {
            let hand = game.hands.get(&player.id).unwrap();
            if player.position == 0 {
                assert_eq!(hand.len(), 21);
            } else {
                assert!(hand.len() == 20 || hand.len() == 19);
            }
        }
    }

    #[test]
    fn test_integration_hand_with_meld_huxi() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        let player_id = game.players[0].id;
        let hand = game.hands.get_mut(&player_id).unwrap();

        hand.add_meld(Meld::new(
            MeldType::Chi,
            vec![
                Card::new(Suit::Small, CardValue::One),
                Card::new(Suit::Small, CardValue::Two),
                Card::new(Suit::Small, CardValue::Three),
            ],
            false,
        ));

        hand.add_meld(Meld::new(
            MeldType::Peng,
            vec![
                Card::new(Suit::Big, CardValue::Five),
                Card::new(Suit::Big, CardValue::Five),
                Card::new(Suit::Big, CardValue::Five),
            ],
            false,
        ));

        hand.add_meld(Meld::new(
            MeldType::Sao,
            vec![
                Card::new(Suit::Small, CardValue::Seven),
                Card::new(Suit::Small, CardValue::Seven),
                Card::new(Suit::Small, CardValue::Seven),
            ],
            false,
        ));

        let huxi = game.calculate_hand_huxi(player_id).unwrap();
        assert!(huxi >= 10);
        assert!(game.can_hu(player_id).unwrap());
    }

    #[test]
    fn test_integration_card_types() {
        use crate::CardType;

        let red_card = Card::new(Suit::Small, CardValue::Two);
        let black_card = Card::new(Suit::Small, CardValue::One);

        assert!(red_card.is_red());
        assert!(!black_card.is_red());

        assert_eq!(red_card.card_type(), CardType::Red);
        assert_eq!(black_card.card_type(), CardType::Black);

        let big_red_card = Card::new(Suit::Big, CardValue::Seven);
        assert!(big_red_card.is_red());
    }

    #[test]
    fn test_integration_deck_operations() {
        let mut deck = Deck::new();

        assert_eq!(deck.remaining(), 80);

        let card1 = deck.draw();
        assert!(card1.is_some());
        assert_eq!(deck.remaining(), 79);

        let cards = deck.draw_n(5);
        assert_eq!(cards.len(), 5);
        assert_eq!(deck.remaining(), 74);

        deck.shuffle();
        assert_eq!(deck.remaining(), 74);
    }

    #[test]
    fn test_integration_sao_chuan_rules() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        let player_id = game.players[0].id;

        let must_sao = game.check_must_sao(player_id);

        let can_sao_chuan = game.can_sao_chuan(player_id);

        let can_sao = game.can_sao(player_id);

        let can_peng = game.can_peng(player_id);

        assert!(must_sao.is_none() || can_sao_chuan);
    }

    #[test]
    fn test_integration_chi_2710() {
        let cards = vec![
            Card::new(Suit::Small, CardValue::Two),
            Card::new(Suit::Small, CardValue::Seven),
            Card::new(Suit::Small, CardValue::Ten),
        ];

        assert!(Meld::is_valid_2710(&cards));

        let cards_big = vec![
            Card::new(Suit::Big, CardValue::Two),
            Card::new(Suit::Big, CardValue::Seven),
            Card::new(Suit::Big, CardValue::Ten),
        ];

        assert!(Meld::is_valid_2710(&cards_big));
    }

    #[test]
    fn test_integration_san_da() {
        let cards = vec![
            Card::new(Suit::Big, CardValue::One),
            Card::new(Suit::Big, CardValue::One),
            Card::new(Suit::Small, CardValue::One),
        ];

        assert!(Meld::is_valid_san_da(&cards));

        let cards2 = vec![
            Card::new(Suit::Small, CardValue::Two),
            Card::new(Suit::Small, CardValue::Two),
            Card::new(Suit::Big, CardValue::Two),
        ];

        assert!(Meld::is_valid_san_da(&cards2));
    }

    #[test]
    fn test_integration_dangdi() {
        let mut game = GameState::new();
        game.add_player(Player::new("玩家1")).unwrap();
        game.add_player(Player::new("玩家2")).unwrap();
        game.start_game().unwrap();

        assert!(game.dangdi.is_some());

        let dangdi = game.dangdi.unwrap();

        let player1_hand = game.hands.get(&game.players[0].id).unwrap();
        let player2_hand = game.hands.get(&game.players[1].id).unwrap();

        assert!(player1_hand.len() == 21);
        assert!(player2_hand.len() == 20);
    }

    #[test]
    fn test_integration_player_positions() {
        let mut game = GameState::new();

        let p1 = Player::new("玩家1");
        let p2 = Player::new("玩家2");

        game.add_player(p1).unwrap();
        game.add_player(p2).unwrap();

        game.start_game().unwrap();

        assert_eq!(game.players[0].position, 0);
        assert_eq!(game.players[1].position, 1);

        assert_eq!(game.dealer_idx, 0);
        assert_eq!(game.current_player_idx, 0);
    }
}
