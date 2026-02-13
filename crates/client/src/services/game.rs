use guilin_paizi_core::{GameState, PlayerId};
use std::rc::Rc;

pub struct GameService {
    pub player_id: Option<PlayerId>,
    pub game_state: Option<GameState>,
}

impl GameService {
    pub fn new() -> Self {
        Self {
            player_id: None,
            game_state: None,
        }
    }

    pub fn is_my_turn(&self) -> bool {
        if let (Some(pid), Some(state)) = (self.player_id, self.game_state.as_ref()) {
            state.get_current_player_id() == Some(pid)
        } else {
            false
        }
    }
}

impl Default for GameService {
    fn default() -> Self {
        Self::new()
    }
}
