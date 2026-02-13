use yew::prelude::*;
use yew_router::prelude::*;
use crate::Route;

#[derive(Properties, PartialEq)]
pub struct RoomPageProps {
    pub room_id: String,
}

#[function_component(RoomPage)]
pub fn room_page(props: &RoomPageProps) -> Html {
    let navigator = use_navigator().unwrap();
    let room_id = props.room_id.clone();

    let start_game = {
        let navigator = navigator.clone();
        let room_id = room_id.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Game { id: room_id.clone() });
        })
    };

    let leave_room = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div class="room-page">
            <header class="room-header">
                <h2>{format!("房间: {}", props.room_id)}</h2>
                <button class="btn-leave" onclick={leave_room}>{"离开"}</button>
            </header>
            
            <div class="room-content">
                <div class="players-list">
                    <h3>{"玩家列表"}</h3>
                    <div class="player-slot">
                        <span class="player-name">{"玩家1 (你)"}</span>
                        <span class="status ready">{"已准备"}</span>
                    </div>
                    <div class="player-slot empty">
                        <span>{"等待加入..."}</span>
                    </div>
                    <div class="player-slot empty">
                        <span>{"等待加入..."}</span>
                    </div>
                </div>
                
                <div class="room-settings">
                    <h3>{"房间设置"}</h3>
                    <div class="setting">
                        <label>{"底分:"}</label>
                        <select>
                            <option value="100">{"100"}</option>
                            <option value="500">{"500"}</option>
                            <option value="1000">{"1000"}</option>
                        </select>
                    </div>
                    <div class="setting">
                        <label>{"技能模式:"}</label>
                        <input type="checkbox" checked={true} />
                    </div>
                </div>
            </div>
            
            <div class="room-actions">
                <button class="btn-ready">{"准备"}</button>
                <button class="btn-start" onclick={start_game}>{"开始游戏"}</button>
            </div>
        </div>
    }
}
