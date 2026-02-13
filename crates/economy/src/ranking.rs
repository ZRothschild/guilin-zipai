use guilin_paizi_core::PlayerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
    Master,
    GrandMaster,
}

impl Tier {
    pub fn stars_to_promote(&self) -> u8 {
        match self {
            Tier::Bronze => 5,
            Tier::Silver => 5,
            Tier::Gold => 5,
            Tier::Platinum => 5,
            Tier::Diamond => 5,
            Tier::Master => 5,
            Tier::GrandMaster => 0,
        }
    }

    pub fn base_rating(&self) -> u32 {
        match self {
            Tier::Bronze => 0,
            Tier::Silver => 500,
            Tier::Gold => 1000,
            Tier::Platinum => 1500,
            Tier::Diamond => 2000,
            Tier::Master => 2500,
            Tier::GrandMaster => 3000,
        }
    }
}

impl std::fmt::Display for Tier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Tier::Bronze => "青铜",
            Tier::Silver => "白银",
            Tier::Gold => "黄金",
            Tier::Platinum => "铂金",
            Tier::Diamond => "钻石",
            Tier::Master => "大师",
            Tier::GrandMaster => "王者",
        };
        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rank {
    pub tier: Tier,
    pub stars: u8,
}

impl Rank {
    pub fn new(tier: Tier, stars: u8) -> Self {
        Self { tier, stars }
    }

    pub fn promote(&mut self) {
        let required = self.tier.stars_to_promote();
        if self.stars >= required && self.tier != Tier::GrandMaster {
            self.tier = match self.tier {
                Tier::Bronze => Tier::Silver,
                Tier::Silver => Tier::Gold,
                Tier::Gold => Tier::Platinum,
                Tier::Platinum => Tier::Diamond,
                Tier::Diamond => Tier::Master,
                Tier::Master => Tier::GrandMaster,
                Tier::GrandMaster => Tier::GrandMaster,
            };
            self.stars = 0;
        }
    }

    pub fn demote(&mut self) {
        if self.stars == 0 && self.tier != Tier::Bronze {
            self.tier = match self.tier {
                Tier::Bronze => Tier::Bronze,
                Tier::Silver => Tier::Bronze,
                Tier::Gold => Tier::Silver,
                Tier::Platinum => Tier::Gold,
                Tier::Diamond => Tier::Platinum,
                Tier::Master => Tier::Diamond,
                Tier::GrandMaster => Tier::Master,
            };
            self.stars = self.tier.stars_to_promote() - 1;
        } else if self.stars > 0 {
            self.stars -= 1;
        }
    }

    pub fn add_star(&mut self) {
        self.stars += 1;
        self.promote();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EloRating {
    pub rating: u32,
    pub games_played: u32,
    pub wins: u32,
    pub losses: u32,
}

impl EloRating {
    pub fn new() -> Self {
        Self {
            rating: 1000,
            games_played: 0,
            wins: 0,
            losses: 0,
        }
    }

    pub fn k_factor(&self) -> u32 {
        if self.games_played < 30 {
            32
        } else if self.rating < 2000 {
            24
        } else {
            16
        }
    }

    pub fn expected_score(&self, opponent_rating: u32) -> f64 {
        let diff = opponent_rating as f64 - self.rating as f64;
        1.0 / (1.0 + 10f64.powf(diff / 400.0))
    }

    pub fn update_rating(&mut self, opponent_rating: u32, won: bool) {
        let k = self.k_factor() as f64;
        let expected = self.expected_score(opponent_rating);
        let actual = if won { 1.0 } else { 0.0 };
        
        let change = (k * (actual - expected)) as i32;
        self.rating = (self.rating as i32 + change).max(0) as u32;
        
        self.games_played += 1;
        if won {
            self.wins += 1;
        } else {
            self.losses += 1;
        }
    }

    pub fn win_rate(&self) -> f64 {
        if self.games_played == 0 {
            0.0
        } else {
            self.wins as f64 / self.games_played as f64
        }
    }
}

impl Default for EloRating {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RankingSystem {
    player_ranks: HashMap<PlayerId, Rank>,
    player_ratings: HashMap<PlayerId, EloRating>,
}

impl RankingSystem {
    pub fn new() -> Self {
        Self {
            player_ranks: HashMap::new(),
            player_ratings: HashMap::new(),
        }
    }

    pub fn register_player(&mut self, player_id: PlayerId) {
        self.player_ranks.insert(player_id, Rank::new(Tier::Bronze, 0));
        self.player_ratings.insert(player_id, EloRating::new());
    }

    pub fn get_rank(&self, player_id: PlayerId) -> Option<&Rank> {
        self.player_ranks.get(&player_id)
    }

    pub fn get_rating(&self, player_id: PlayerId) -> Option<&EloRating> {
        self.player_ratings.get(&player_id)
    }

    pub fn update_after_match(&mut self, winner: PlayerId, loser: PlayerId) {
        let winner_rating = self.player_ratings.get(&winner).map(|r| r.rating).unwrap_or(1000);
        let loser_rating = self.player_ratings.get(&loser).map(|r| r.rating).unwrap_or(1000);

        if let Some(rating) = self.player_ratings.get_mut(&winner) {
            rating.update_rating(loser_rating, true);
        }

        if let Some(rating) = self.player_ratings.get_mut(&loser) {
            rating.update_rating(winner_rating, false);
        }

        if let Some(rank) = self.player_ranks.get_mut(&winner) {
            rank.add_star();
        }

        if let Some(rank) = self.player_ranks.get_mut(&loser) {
            rank.demote();
        }
    }
}

impl Default for RankingSystem {
    fn default() -> Self {
        Self::new()
    }
}
