use yew::prelude::*;
use yew_router::prelude::*;
use crate::{Route, GameContext};

#[derive(Properties, PartialEq)]
pub struct RoomPageProps {
    pub room_id: String,
}

#[function_component(RoomPage)]
pub fn room_page(props: &RoomPageProps) -> Html {
    let navigator = use_navigator().unwrap();
    let context = use_context::<GameContext>().expect("No GameContext found");
    let room_id = props.room_id.clone();

    let start_game = {
        let send_message = context.send_message.clone();
        let room_id = room_id.clone();
        Callback::from(move |_| {
            send_message.emit(serde_json::json!({
                "type": "StartGame",
                "room_id": room_id
            }));
        })
    };

    let leave_room = {
        let send_message = context.send_message.clone();
        let room_id = room_id.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            send_message.emit(serde_json::json!({
                "type": "LeaveRoom",
                "room_id": room_id
            }));
            navigator.push(&Route::Home);
        })
    };

    let set_ready = {
        let send_message = context.send_message.clone();
        let room_id = room_id.clone();
        Callback::from(move |_| {
            send_message.emit(serde_json::json!({
                "type": "Ready",
                "room_id": room_id
            }));
        })
    };

    let add_bot = {
        let send_message = context.send_message.clone();
        let room_id = room_id.clone();
        Callback::from(move |_| {
            send_message.emit(serde_json::json!({
                "type": "AddBot",
                "room_id": room_id
            }));
        })
    };

    let players_html = context.state.players.iter().map(|p| {
        let is_me = context.state.local_player_id == Some(p.id);
        html! {
            <div class="player-slot">
                <span class="player-name">
                    {format!("{} {}", p.name, if is_me { "(你)" } else { "" })}
                    {if p.is_bot { " [机器人]" } else { "" }}
                </span>
                <span class={if p.is_ready { "status ready" } else { "status waiting" }}>
                    {if p.is_ready { "已准备" } else { "准备中" }}
                </span>
            </div>
        }
    }).collect::<Html>();

    // 填充空白槽位
    let empty_slots = (context.state.players.len()..3).map(|_| {
        html! {
            <div class="player-slot empty">
                <span>{"等待加入..."}</span>
            </div>
        }
    }).collect::<Html>();

    html! {
        <div class="room-page">
            <header class="room-header">
                <h2>{format!("房间: {}", props.room_id)}</h2>
                <button class="btn-leave" onclick={leave_room}>{"离开"}</button>
            </header>
            
            <div class="room-content">
                <div class="players-list">
                    <h3>{"玩家列表"}</h3>
                    {players_html}
                    {empty_slots}
                </div>
                
                <div class="room-settings">
                    <h3>{"功能区"}</h3>
                    <button class="btn-secondary" onclick={add_bot}>{"邀请机器人"}</button>
                    <div class="setting-info">
                        <p>{"桂林字牌规则：满3人或2人以上准备可开始"}</p>
                    </div>
                </div>
            </div>
            
            <div class="room-actions">
                <button class="btn-ready" onclick={set_ready}>{"准备"}</button>
                <button class="btn-start" onclick={start_game} disabled={context.state.players.len() < 2}>
                    {"开始游戏"}
                </button>
            </div>
        </div>
    }
}
