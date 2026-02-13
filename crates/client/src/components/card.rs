use yew::prelude::*;
use guilin_paizi_core::Card;

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub card: Card,
    #[prop_or_default]
    pub on_click: Option<Callback<MouseEvent>>,
    #[prop_or(false)]
    pub selected: bool,
    #[prop_or(false)]
    pub disabled: bool,
}

#[function_component(CardView)]
pub fn card_view(props: &CardProps) -> Html {
    let card_class = classes!(
        "card",
        if props.card.is_red() { "red" } else { "black" },
        if props.selected { "selected" } else { "" },
        if props.disabled { "disabled" } else { "" },
    );

    let suit_class = if props.card.suit == guilin_paizi_core::Suit::Big { "big" } else { "small" };

    html! {
        <div 
            class={card_class}
            onclick={props.on_click.clone()}
        >
            <span class={classes!("suit", suit_class)}>
                {props.card.to_string()}
            </span>
        </div>
    }
}
