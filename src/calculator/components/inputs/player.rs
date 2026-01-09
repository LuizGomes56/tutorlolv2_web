use crate::{
    calculator::{
        Player,
        components::inputs::{
            abilities::Abilities,
            banner::Banner,
            stats::{StatCell, Stats},
        },
        page::PlayerProps,
        reducer::{DataAction, LastAction, PlayerAction},
    },
    components::image::Image,
    model::PlayerStats,
    utils::ImageType,
};
use std::{cell::RefCell, rc::Rc};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[hook]
fn use_player_callback<T: 'static>(
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

#[hook]
fn use_data_callback<T: 'static>(
    props: &PlayerProps,
    callback: fn(T) -> DataAction<PlayerStats>,
) -> Callback<T> {
    let PlayerProps {
        player,
        last_action,
    } = props.clone();
    use_callback((), move |v, _| {
        let value = callback(v);
        last_action.replace(LastAction::CurrentPlayer);
        player.dispatch(PlayerAction::Data(value));
    })
}

#[derive(PartialEq, Properties)]
pub struct PlayerInputProps {
    pub player_props: PlayerProps,
}

#[component]
pub fn PlayerInput(props: &PlayerInputProps) -> Html {
    let PlayerInputProps { player_props } = props;

    let player = &player_props.player;
    let data = &player.data;

    let stats_cb = use_data_callback(player_props, DataAction::Stats);
    let level_cb = use_data_callback(player_props, DataAction::Level);
    let abilities_cb = use_player_callback(player_props, PlayerAction::AbilityLevels);

    html! {
        <div class={classes!("flex", "flex-col", "w-72", "box")}>
            <Banner champion_id={data.champion_id} />
            <div class={classes!("grid", "grid-cols-4")}>
                <Abilities
                    ability_levels={player.abilities}
                    callback={abilities_cb}
                    champion_id={data.champion_id}
                />
            </div>
            <div class={classes!(
                "grid", "grid-cols-[auto,1fr,1fr]",
                "gap-x-2", "p-4", "oxanium"
            )}>
                <StatCell
                    image_type={ImageType::Level}
                    name={"Level"}
                    disabled={false}
                    value={data.level as i32}
                    placeholder={1}
                    oninput={{
                        let callback = level_cb.clone();
                        Callback::from(move |e: InputEvent| {
                            let value = e.target_unchecked_into::<HtmlInputElement>().value();
                            let number = value.parse().unwrap_or(1);
                            callback.emit(number);
                        })
                    }}
                />
                <Stats
                    infer={data.infer_stats}
                    stats={data.stats}
                    callback={stats_cb}
                />
            </div>
        </div>
    }
}
