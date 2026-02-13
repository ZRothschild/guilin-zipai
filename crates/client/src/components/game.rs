use yew::prelude::*;
use guilin_paizi_core::GameState;
use super::{HandView, PlayerInfo};

#[derive(Properties, PartialEq)]
pub struct GameBoardProps {
    pub game_state: GameState,
    pub current_player_id: Option<guilin_paizi_core::PlayerId>,
}

#[function_component(GameBoard)]
pub fn game_board(props: &GameBoardProps) -> Html {
    let players_html: Html = props.game_state.players.iter().map(|player| {
        let is_current = props.current_player_id == Some(player.id);
        let card_count = props.game_state.hands.get(&player.id)
            .map(|h| h.len())
            .unwrap_or(0);

        html! {
            <PlayerInfo 
                player={player.clone()} 
                is_current={is_current}
                card_count={card_count}
            />
        }
    }).collect();

    let discard_html: Html = props.game_state.discard_pile.iter().rev().take(5).map(|(_, card)| {
        html! {
            <div class="discard-card">{card.to_string()}</div>
        }
    }).collect();

    html! {
        <div class="game-board">
            <div class="players-area">
                {players_html}
            </div>
            
            <div class="table-center">
                <div class="deck-info">
                    {format!("剩余: {} 张", props.game_state.deck.remaining())}
                </div>
                
                <div class="discard-pile">
                    {discard_html}
                </div>
                
                if let Some(dangdi) = props.game_state.dangdi {
                    <div class="dangdi">
                        {format!("挡底: {}", dangdi)}
                    </div>
                }
            </div>
            
            <div class="action-buttons">
                <button class="btn-action">{"吃"}</button>
                <button class="btn-action">{"碰"}</button>
                <button class="btn-action">{"胡"}</button>
                <button class="btn-action">{"过"}</button>
            </div>
        </div>
    }
}
