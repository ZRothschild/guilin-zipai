use guilin_paizi_core::{PlayerId, GameState};
use yew::prelude::*;
use crate::models::PlayerInfo;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AppState {
    pub local_player_id: Option<PlayerId>,
    pub current_room: Option<String>,
    pub players: Vec<PlayerInfo>,
    pub happy_beans: u64,
    pub notifications: Vec<String>,
    pub is_gaming: bool,
    pub game_state: GameState,
}

pub enum AppAction {
    SetLocalPlayer(PlayerId),
    RoomJoined(String),
    UpdatePlayers(Vec<PlayerInfo>),
    AddNotification(String),
    StartGame,
    UpdateGameState(GameState),
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        let mut next = (*self).clone();
        match action {
            AppAction::SetLocalPlayer(id) => next.local_player_id = Some(id),
            AppAction::RoomJoined(id) => {
                next.current_room = Some(id);
                next.is_gaming = false;
            },
            AppAction::UpdatePlayers(players) => next.players = players,
            AppAction::AddNotification(msg) => {
                next.notifications.push(msg);
                if next.notifications.len() > 10 { next.notifications.remove(0); }
            },
            AppAction::StartGame => next.is_gaming = true,
            AppAction::UpdateGameState(state) => next.game_state = state,
        }
        next.into()
    }
}
