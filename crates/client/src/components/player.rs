use yew::prelude::*;
use guilin_paizi_core::player::{Player, PlayerState};

#[derive(Properties, PartialEq)]
pub struct PlayerProps {
    pub player: Player,
    pub is_current: bool,
    pub card_count: usize,
}

#[function_component(PlayerInfo)]
pub fn player_info(props: &PlayerProps) -> Html {
    let status_class = match props.player.state {
        PlayerState::Ready => "ready",
        PlayerState::Playing => "playing",
        _ => "waiting",
    };

    html! {
        <div class={classes!("player-info", if props.is_current { "current" } else { "" })}>
            <div class="player-avatar"></div>
            <div class="player-name">{&props.player.name}</div>
            <div class={classes!("player-status", status_class)}>
                {format!("{} 张", props.card_count)}
            </div>
            if props.player.is_dealer {
                <div class="dealer-badge">{"庄"}</div>
            }
        </div>
    }
}
