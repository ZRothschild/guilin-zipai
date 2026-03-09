use yew::prelude::*;
use yew_router::prelude::*;
use crate::{Route, GameContext};

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let navigator = use_navigator().unwrap();
    let context = use_context::<GameContext>().expect("No GameContext found");

    let create_room = {
        let send_message = context.send_message.clone();
        Callback::from(move |_| {
            send_message.emit(serde_json::json!({
                "type": "CreateRoom",
                "max_players": 3
            }));
        })
    };

    // 监听状态，当加入房间成功时自动跳转
    {
        let navigator = navigator.clone();
        let current_room = context.state.current_room.clone();
        use_effect_with(current_room, move |room| {
            if let Some(id) = room {
                navigator.push(&Route::Room { id: id.clone() });
            }
            move || {}
        });
    }

    html! {
        <div class="home-page">
            <header class="game-header">
                <h1>{"桂林字牌"}</h1>
                <p class="subtitle">{"Guilin Paizi - 广西特色纸牌游戏"}</p>
            </header>
            
            <main class="main-menu">
                <div class="menu-card">
                    <h2>{"快速开始"}</h2>
                    <button class="btn-primary" onclick={create_room}>
                        {"创建房间"}
                    </button>
                    <button class="btn-secondary">
                        {"加入房间"}
                    </button>
                </div>
                
                <div class="menu-card">
                    <h2>{"游戏模式"}</h2>
                    <div class="mode-buttons">
                        <button class="btn-mode">{"休闲赛"}</button>
                        <button class="btn-mode">{"段位赛"}</button>
                        <button class="btn-mode">{"锦标赛"}</button>
                    </div>
                </div>
                
                <div class="menu-card">
                    <h2>{"个人信息"}</h2>
                    <div class="player-stats">
                        <div class="stat">
                            <span class="stat-label">{"欢乐豆:"}</span>
                            <span class="stat-value">{"10,000"}</span>
                        </div>
                        <div class="stat">
                            <span class="stat-label">{"段位:"}</span>
                            <span class="stat-value">{"青铜 I"}</span>
                        </div>
                    </div>
                </div>
            </main>
            
            <footer class="game-footer">
                <p>{"© 2024 桂林字牌 - 传承广西文化"}</p>
            </footer>
        </div>
    }
}
