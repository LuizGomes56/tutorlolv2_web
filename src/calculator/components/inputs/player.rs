use crate::{
    calculator::{
        components::inputs::{
            _item_selector::ItemSelector,
            abilities::Abilities,
            banner::Banner,
            exceptions::Exceptions,
            item_selector::ItemButton,
            recommendations::Recommendations,
            stats::{StatCell, Stats},
            tray::Tray,
        },
        page::{PlayerProps, TargetEntity},
        reducer::{DataAction, LastAction, PlayerAction},
    },
    components::image::ImageType,
    model::PlayerStats,
    utils::traits::Print,
};
use tutorlolv2_gen::{ItemId, RuneId};
use web_sys::HtmlInputElement;
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
        last_action.replace(LastAction::CurrentPlayer);
        player.dispatch(value);
    })
}

#[hook]
pub fn use_data_callback<T: 'static>(
    props: &PlayerProps,
    callback: fn(T) -> DataAction<PlayerStats>,
) -> Callback<T> {
    let PlayerProps {
        player,
        last_action,
    } = props.clone();
    use_callback((), move |v, _| {
        let value = callback(v);
        last_action.replace(value.action(LastAction::CurrentPlayer));
        player.dispatch(PlayerAction::Data(value));
    })
}

#[derive(PartialEq, Properties)]
pub struct PlayerInputProps {
    pub player_props: PlayerProps,
    pub open_item_menu: Callback<TargetEntity>,
}

#[component]
pub fn PlayerInput(props: &PlayerInputProps) -> Html {
    let PlayerInputProps {
        player_props,
        open_item_menu,
    } = props;

    let player = &player_props.player;
    let data = &player.data;

    let stats_cb = use_data_callback(player_props, DataAction::Stats);
    let level_cb = use_data_callback(player_props, DataAction::Level);
    let abilities_cb = use_player_callback(player_props, PlayerAction::AbilityLevels);
    let champion_cb = use_data_callback(player_props, DataAction::ChampionId);

    let insert_rune = use_player_callback(player_props, PlayerAction::InsertRune);
    let remove_rune = use_player_callback(player_props, PlayerAction::RemoveRune);
    let recommended_runes = use_player_callback(player_props, PlayerAction::SetRuneVec);

    let item_exception_callback = use_data_callback(player_props, DataAction::InsertItemExc);
    let rune_exception_callback = use_player_callback(player_props, PlayerAction::InsertRuneExc);
    let stack_callback = use_data_callback(player_props, DataAction::Stacks);

    html! {
        <>
            <div class={classes!("flex", "flex-col", "w-64", "box", "m-2")}>
                <Banner
                    callback={champion_cb}
                    champion_id={data.champion_id}
                />
                <div class={classes!("grid", "grid-cols-4")}>
                    <Abilities
                        ability_levels={player.abilities}
                        callback={abilities_cb}
                        champion_id={data.champion_id}
                    />
                </div>
                <ItemButton
                    onclick={{
                        let open_item_menu = open_item_menu.clone();
                        Callback::from(move |_| {
                            open_item_menu.emit(TargetEntity::Player);
                        })
                    }}
                    length={player.data.items.len()}
                />
                <Exceptions
                    items={player.data.items.clone()}
                    runes={player.runes.clone()}
                    item_exceptions={player.data.item_exceptions.clone()}
                    rune_exceptions={player.rune_exceptions.clone()}
                    item_callback={item_exception_callback}
                    rune_callback={rune_exception_callback}
                    stack_callback={stack_callback}
                    stacks={player.data.stacks}
                    champion_id={player.data.champion_id}
                />
                <div class={classes!(
                    "grid", "grid-cols-[auto,1fr,1fr]",
                    "gap-x-2", "px-4", "py-3", "gap-y-0.5"
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
                    <Stats<PlayerStats>
                        infer={data.infer_stats}
                        stats={data.stats}
                        callback={stats_cb}
                    />
                </div>
            </div>
        </>
    }
}
