use yew::prelude::*;
use guilin_paizi_core::{Hand};
use crate::components::{GameBoard, HandView};
use crate::GameContext;

#[derive(Properties, PartialEq)]
pub struct GamePageProps {
    pub room_id: String,
}

#[function_component(GamePage)]
pub fn game_page(props: &GamePageProps) -> Html {
    let context = use_context::<GameContext>().expect("No GameContext found");
    let state = &context.state;
    
    let local_player_id = state.local_player_id;
    let game_state = &state.game_state;
    
    let current_hand = if let Some(id) = local_player_id {
        game_state.hands.get(&id).cloned().unwrap_or_else(|| Hand::new(vec![]))
    } else {
        Hand::new(vec![])
    };

    let selected_cards = use_state(|| Vec::<usize>::new());

    let on_card_click = {
        let selected = selected_cards.clone();
        let hand_len = current_hand.cards().len();
        Callback::from(move |idx: usize| {
            if idx >= hand_len { return; }
            let mut new_selection = (*selected).clone();
            if let Some(pos) = new_selection.iter().position(|&x| x == idx) {
                new_selection.remove(pos);
            } else {
                new_selection.push(idx);
            }
            selected.set(new_selection);
        })
    };

    let play_selected = {
        let send_message = context.send_message.clone();
        let selected = selected_cards.clone();
        let room_id = props.room_id.clone(); // Use props.room_id directly
        let local_player_id = local_player_id; // Capture local_player_id
        Callback::from(move |_: MouseEvent| {
            if let Some(&card_idx) = selected.first() {
                if let Some(player_id) = local_player_id {
                    send_message.emit(serde_json::json!({
                        "type": "PlayCard",
                        "room_id": room_id,
                        "player_id": player_id, // Include player_id
                        "card_idx": card_idx
                    }));
                    selected.set(vec![]);
                }
            }
        })
    };

    html! {
        <div class="game-page">
            <header class="game-header">
                <h3>{format!("对局中 - {}", props.room_id)}</h3>
                <div class="game-info">
                    <span>{format!("相位: {:?}", game_state.phase)}</span>
                    <span>{format!("牌堆剩余: {} 张", game_state.deck.remaining())}</span>
                </div>
            </header>
            
            <GameBoard 
                game_state={game_state.clone()}
                current_player_id={local_player_id}
            />
            <div class="player-area">
                <div class="action-panel">
                    <HandView 
                        hand={current_hand}
                        on_card_click={Some(on_card_click)}
                        selected_cards={(*selected_cards).clone()}
                    />
                    <button class="btn-play" onclick={play_selected} disabled={selected_cards.is_empty()}>
                        {"出牌"}
                    </button>
                </div>
                
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
