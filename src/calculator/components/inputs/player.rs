use crate::calculator::{
    Player,
    page::PlayerProps,
    reducer::{LastAction, PlayerAction},
};
use std::{cell::RefCell, rc::Rc};
use yew::prelude::*;

#[hook]
pub fn use_player_callback<T: 'static>(
    props: &PlayerProps,
    callback: fn(T) -> PlayerAction,
) -> Callback<T> {
    let PlayerProps {
        player,
        last_action,
    } = props.clone();
    use_callback((), move |v, _| {
        let value = callback(v);
        last_action.replace(value.action());
        player.dispatch(value);
    })
}

#[derive(PartialEq, Properties)]
pub struct PlayerInputProps {
    pub player_props: PlayerProps,
}

#[component]
pub fn PlayerInput(props: &PlayerInputProps) -> Html {
    let PlayerInputProps { player_props } = props;
    html! {
        <div></div>
    }
}
