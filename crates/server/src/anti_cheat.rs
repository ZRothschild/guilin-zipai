use guilin_paizi_core::{PlayerId, GameState, GameAction};
use std::collections::{HashMap, VecDeque};

pub struct AntiCheatSystem {
    action_history: HashMap<PlayerId, VecDeque<GameAction>>,
    suspicious_patterns: Vec<SuspiciousPattern>,
}

#[derive(Debug, Clone)]
pub struct SuspiciousPattern {
    pub player_id: PlayerId,
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub enum PatternType {
    ImpossibleWinRate,
    TooFastActions,
    PredictableBehavior,
    MultipleAccounts,
}

impl AntiCheatSystem {
    pub fn new() -> Self {
        Self {
            action_history: HashMap::new(),
            suspicious_patterns: Vec::new(),
        }
    }

    pub fn record_action(&mut self, player_id: PlayerId, action: GameAction) {
        let history = self.action_history.entry(player_id).or_insert_with(|| VecDeque::with_capacity(100));
        history.push_back(action);
        
        if history.len() > 100 {
            history.pop_front();
        }
    }

    pub fn validate_action(&self, game_state: &GameState, player_id: PlayerId, action: &GameAction) -> ValidationResult {
        if let Some(current) = game_state.get_current_player_id() {
            if current != player_id {
                return ValidationResult::Invalid("不是当前玩家的回合".to_string());
            }
        }

        match action {
            GameAction::PlayCard { player, card_idx } => {
                if let Some(hand) = game_state.hands.get(player) {
                    if *card_idx >= hand.cards().len() {
                        return ValidationResult::Invalid("无效的牌索引".to_string());
                    }
                }
            }
            _ => {}
        }

        ValidationResult::Valid
    }

    pub fn check_patterns(&mut self, player_id: PlayerId, _game_state: &GameState) -> Vec<SuspiciousPattern> {
        let mut detected = Vec::new();

        if let Some(history) = self.action_history.get(&player_id) {
            if history.len() >= 10 {
                let recent_actions: Vec<_> = history.iter().rev().take(10).collect();
                
                let all_same_timing = recent_actions.windows(2).all(|w| {
                    true
                });

                if all_same_timing && history.len() >= 20 {
                    detected.push(SuspiciousPattern {
                        player_id,
                        pattern_type: PatternType::TooFastActions,
                        confidence: 0.7,
                        timestamp: std::time::SystemTime::now(),
                    });
                }
            }
        }

        for pattern in &detected {
            self.suspicious_patterns.push(pattern.clone());
        }

        detected
    }

    pub fn get_suspicious_players(&self) -> Vec<PlayerId> {
        let mut players: Vec<_> = self.suspicious_patterns.iter()
            .map(|p| p.player_id)
            .collect();
        players.dedup();
        players
    }
}

pub enum ValidationResult {
    Valid,
    Invalid(String),
    Suspicious(String),
}

impl Default for AntiCheatSystem {
    fn default() -> Self {
        Self::new()
    }
}
