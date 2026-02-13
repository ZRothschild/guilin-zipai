use yew::prelude::*;
use guilin_paizi_core::{GameState, Hand, Player};
use crate::components::{GameBoard, HandView};

#[derive(Properties, PartialEq)]
pub struct GamePageProps {
    pub room_id: String,
}

#[function_component(GamePage)]
pub fn game_page(props: &GamePageProps) -> Html {
    let game_state = use_state(|| {
        let mut state = GameState::new();
        
        let p1 = Player::new("玩家1");
        let p2 = Player::new("玩家2");
        state.add_player(p1).ok();
        state.add_player(p2).ok();
        state.start_game().ok();
        
        state
    });

    let selected_cards = use_state(|| Vec::<usize>::new());

    let on_card_click = {
        let selected = selected_cards.clone();
        Callback::from(move |idx: usize| {
            let mut new_selection = (*selected).clone();
            if let Some(pos) = new_selection.iter().position(|&x| x == idx) {
                new_selection.remove(pos);
            } else {
                new_selection.push(idx);
            }
            selected.set(new_selection);
        })
    };

    let current_hand = game_state.hands.values().next().cloned().unwrap_or_else(|| Hand::new(vec![]));

    html! {
        <div class="game-page">
            <header class="game-header">
                <h3>{format!("对局中 - {}", props.room_id)}</h3>
                <div class="game-info">
                    <span>{"第 1 局"}</span>
                    <span>{"剩余 55 张"}</span>
                </div>
            </header>
            
            <GameBoard 
                game_state={(*game_state).clone()}
                current_player_id={None}
            />
            
            <div class="player-area">
                <HandView 
                    hand={current_hand}
                    on_card_click={Some(on_card_click)}
                    selected_cards={(*selected_cards).clone()}
                />
                
                <div class="skill-bar">
                    <h4>{"技能"}</h4>
                    <div class="skills">
                        <button class="btn-skill">{"观流"}</button>
                        <button class="btn-skill">{"稳手"}</button>
                        <button class="btn-skill">{"加码"}</button>
                    </div>
                </div>
            </div>
        </div>
    }
}
