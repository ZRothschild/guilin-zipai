mod components;
mod pages;
mod services;
mod state;
mod models;

use yew::prelude::*;
use yew::html::ChildrenProps;
use yew_router::prelude::*;
use pages::{HomePage, GamePage, RoomPage};
use crate::services::websocket::WebSocketService;
use state::{AppState, AppAction};
use std::rc::Rc;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/room/:id")]
    Room { id: String },
    #[at("/game/:id")]
    Game { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, PartialEq)]
pub struct GameContext {
    pub state: Rc<AppState>,
    pub send_message: Callback<serde_json::Value>,
}

#[function_component(GameProvider)]
pub fn game_provider(props: &ChildrenProps) -> Html {
    let state = use_reducer(AppState::default);
    let ws = use_mut_ref(|| WebSocketService::new());
    let navigator = use_navigator();
    
    let dispatcher = state.dispatcher();
    
    {
        let dispatcher = dispatcher.clone();
        let ws = ws.clone();
        use_effect_with((), move |_| {
            let mut ws_service = ws.borrow_mut();
            log::info!("Attempting to connect to WebSocket at ws://127.0.0.1:8080");
            let _ = ws_service.connect_with_cb("ws://127.0.0.1:8080", move |msg| {
                log::info!("Received message: {}", msg);
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&msg) {
                    match value["type"].as_str() {
                        Some("Welcome") => {
                            log::info!("Welcome message received");
                            if let Ok(id) = serde_json::from_value(value["player_id"].clone()) {
                                dispatcher.dispatch(AppAction::SetLocalPlayer(id));
                            }
                        }
                        Some("RoomJoined") | Some("RoomCreated") => {
                            if let Some(room_id) = value["room_id"].as_str() {
                                log::info!("Room joined: {}", room_id);
                                dispatcher.dispatch(AppAction::RoomJoined(room_id.to_string()));
                            }
                        }
                        Some("RoomUpdate") => {
                            if let Some(room_info_str) = value["room_info"].as_str() {
                                match serde_json::from_str::<crate::models::RoomInfo>(room_info_str) {
                                    Ok(room_info) => {
                                        log::info!("Room info updated: {} players", room_info.players.len());
                                        dispatcher.dispatch(AppAction::UpdatePlayers(room_info.players));
                                    },
                                    Err(e) => log::error!("Failed to deserialize RoomInfo: {:?}", e),
                                }
                            }
                        }
                        Some("GameStateUpdate") => {
                            if let Some(gs_str) = value["state"].as_str() {
                                match serde_json::from_str::<guilin_paizi_core::GameState>(gs_str) {
                                    Ok(gs) => {
                                        log::info!("Game state updated: phase {:?}", gs.phase);
                                        dispatcher.dispatch(AppAction::UpdateGameState(gs));
                                    },
                                    Err(e) => log::error!("Failed to deserialize GameState: {:?}", e),
                                }
                            }
                        }
                        Some("GameStarted") => {
                            log::info!("Game started!");
                            dispatcher.dispatch(AppAction::StartGame);
                        }
                        _ => log::warn!("Unknown message type: {:?}", value["type"]),
                    }
                } else {
                    log::error!("Failed to parse message as JSON: {}", msg);
                }
            });
            
            move || {}
        });
    }

    // 监听 StartGame 状态进行跳转
    let is_gaming = state.is_gaming;
    let room_id = state.current_room.clone();
    {
        let navigator = navigator.clone();
        use_effect_with((is_gaming, room_id), move |(gaming, rid)| {
            if *gaming {
                if let (Some(nav), Some(id)) = (navigator, rid) {
                    nav.push(&Route::Game { id: id.clone() });
                }
            }
            move || {}
        });
    }

    let send_message = {
        let ws = ws.clone();
        Callback::from(move |msg: serde_json::Value| {
            if let Ok(text) = serde_json::to_string(&msg) {
                let _ = ws.borrow().send_text(&text);
            }
        })
    };

    let context = GameContext {
        state: Rc::new((*state).clone()),
        send_message,
    };

    html! {
        <ContextProvider<GameContext> context={context}>
            { props.children.clone() }
        </ContextProvider<GameContext>>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePage /> },
        Route::Room { id } => html! { <RoomPage room_id={id} /> },
        Route::Game { id } => html! { <GamePage room_id={id} /> },
        Route::NotFound => html! { <h1>{"页面未找到"}</h1> },
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <GameProvider>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </GameProvider>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
