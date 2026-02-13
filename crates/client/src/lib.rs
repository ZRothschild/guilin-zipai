mod components;
mod pages;
mod services;
mod state;
mod models;

use yew::prelude::*;
use yew_router::prelude::*;
use pages::{HomePage, GamePage, RoomPage};

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
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
