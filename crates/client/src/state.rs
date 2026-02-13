use guilin_paizi_core::PlayerId;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub current_user: Option<PlayerId>,
    pub current_room: Option<String>,
    pub happy_beans: u64,
    pub notifications: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_notification(&mut self, message: impl Into<String>) {
        self.notifications.push(message.into());
        if self.notifications.len() > 10 {
            self.notifications.remove(0);
        }
    }
}
