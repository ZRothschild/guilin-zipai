use yew::prelude::*;
use guilin_paizi_core::Hand;
use super::card::CardView;

#[derive(Properties, PartialEq)]
pub struct HandProps {
    pub hand: Hand,
    #[prop_or_default]
    pub on_card_click: Option<Callback<usize>>,
    #[prop_or_default]
    pub selected_cards: Vec<usize>,
}

#[function_component(HandView)]
pub fn hand_view(props: &HandProps) -> Html {
    let cards_html: Html = props.hand.cards().iter().enumerate().map(|(idx, card)| {
        let is_selected = props.selected_cards.contains(&idx);
        let on_click = props.on_card_click.clone();
        
        let click_handler = if let Some(callback) = on_click {
            let callback = callback.clone();
            Some(Callback::from(move |_| callback.emit(idx)))
        } else {
            None
        };

        html! {
            <CardView 
                card={*card} 
                selected={is_selected}
                on_click={click_handler}
            />
        }
    }).collect();

    html! {
        <div class="hand">
            {cards_html}
        </div>
    }
}
