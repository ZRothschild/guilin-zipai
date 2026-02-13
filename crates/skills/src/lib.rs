use guilin_paizi_core::{GameState, GameAction, PlayerId};

pub mod skills;
pub mod trigger;
pub mod effect;

pub use skills::*;
pub use trigger::{SkillTrigger, TriggerCondition};
pub use effect::{SkillEffect, EffectResult};

pub trait Skill: Send + Sync {
    fn id(&self) -> u32;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> SkillCategory;
    fn max_uses(&self) -> u8;
    fn can_use(&self, game_state: &GameState, player_id: PlayerId) -> bool;
    fn use_skill(&mut self, game_state: &mut GameState, player_id: PlayerId, target: Option<PlayerId>) -> SkillResult;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCategory {
    Information,
    ErrorCorrection,
    Economy,
    Risk,
}

impl std::fmt::Display for SkillCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkillCategory::Information => write!(f, "信息类"),
            SkillCategory::ErrorCorrection => write!(f, "容错类"),
            SkillCategory::Economy => write!(f, "收益类"),
            SkillCategory::Risk => write!(f, "风险类"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SkillResult {
    pub success: bool,
    pub message: String,
    pub effect_data: Option<serde_json::Value>,
}

impl SkillResult {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            effect_data: None,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            effect_data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.effect_data = Some(data);
        self
    }
}

pub struct SkillInstance {
    pub skill: Box<dyn Skill>,
    pub remaining_uses: u8,
}

impl SkillInstance {
    pub fn new(skill: Box<dyn Skill>) -> Self {
        let uses = skill.max_uses();
        Self {
            skill,
            remaining_uses: uses,
        }
    }

    pub fn try_use(&mut self, game_state: &mut GameState, player_id: PlayerId, target: Option<PlayerId>) -> SkillResult {
        if self.remaining_uses == 0 {
            return SkillResult::failure("技能使用次数已耗尽");
        }

        if !self.skill.can_use(game_state, player_id) {
            return SkillResult::failure("当前无法使用该技能");
        }

        let result = self.skill.use_skill(game_state, player_id, target);
        
        if result.success {
            self.remaining_uses -= 1;
        }

        result
    }
}

pub struct SkillManager {
    player_skills: std::collections::HashMap<PlayerId, Vec<SkillInstance>>,
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            player_skills: std::collections::HashMap::new(),
        }
    }

    pub fn assign_skills(&mut self, player_id: PlayerId, skills: Vec<Box<dyn Skill>>) {
        let instances: Vec<_> = skills.into_iter()
            .map(|s| SkillInstance::new(s))
            .collect();
        self.player_skills.insert(player_id, instances);
    }

    pub fn get_player_skills(&self, player_id: PlayerId) -> Option<&Vec<SkillInstance>> {
        self.player_skills.get(&player_id)
    }

    pub fn get_player_skills_mut(&mut self, player_id: PlayerId) -> Option<&mut Vec<SkillInstance>> {
        self.player_skills.get_mut(&player_id)
    }

    pub fn use_skill(&mut self, player_id: PlayerId, skill_idx: usize, game_state: &mut GameState, target: Option<PlayerId>) -> Option<SkillResult> {
        if let Some(skills) = self.player_skills.get_mut(&player_id) {
            if let Some(instance) = skills.get_mut(skill_idx) {
                return Some(instance.try_use(game_state, player_id, target));
            }
        }
        None
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}
