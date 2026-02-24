use crate::{
    calculator::{
        components::inputs::{
            abilities::Abilities,
            banner::Banner,
            item_selector::ItemSelector,
            recommendations::Recommendations,
            selector::{Selector, item_filter},
            stats::{StatCell, Stats},
            tray::Tray,
        },
        page::PlayerProps,
        reducer::{DataAction, LastAction, PlayerAction},
    },
    components::image::ImageType,
    model::PlayerStats,
};
use tutorlolv2_gen::{ItemId, RuneId};
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
        last_action.replace(LastAction::CurrentPlayer);
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
        last_action.replace(value.action(LastAction::CurrentPlayer));
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
    let champion_cb = use_data_callback(player_props, DataAction::ChampionId);

    let insert_item = use_data_callback(player_props, DataAction::InsertItem);
    let insert_rune = use_player_callback(player_props, PlayerAction::InsertRune);

    let remove_item = use_data_callback(player_props, DataAction::RemoveItem);
    let remove_rune = use_player_callback(player_props, PlayerAction::RemoveRune);

    let recommended_items = use_data_callback(player_props, DataAction::SetItemVec);
    let recommended_runes = use_player_callback(player_props, PlayerAction::SetRuneVec);

    html! {
        <>
            <ItemSelector
                insert={insert_item}
                remove={remove_item}
                recommended={recommended_items}
                items={data.items.clone()}
            />
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
            // <div class={classes!("flex", "flex-col", "w-64", "box", "m-2")}>
            //     <Recommendations
            //         callback={{
            //             let recommended_items = recommended_items.clone();
            //             let champion_id = data.champion_id;
            //             Callback::from(move |position| {
            //                 let rec = champion_id.recommended_items(position);
            //                 recommended_items.emit(rec);
            //             })
            //         }}
            //     />
            //     <Selector<ItemId>
            //         callback={insert_item}
            //         filter={Callback::from(item_filter)}
            //     />
            //     <div class={classes!("bg-emerald-500", "py-2", "my-4")} />
            //     <Tray<ItemId> callback={remove_item} vector={data.items.clone()} />

            //     <Recommendations
            //         callback={{
            //             let recommended_runes = recommended_runes.clone();
            //             let champion_id = data.champion_id;
            //             Callback::from(move |position| {
            //                 let rec = champion_id.recommended_runes(position);
            //                 recommended_runes.emit(rec);
            //             })
            //         }}
            //     />
            //     <Selector<RuneId> callback={insert_rune} />
            //     <div class={classes!("bg-emerald-500", "py-2", "my-4")} />
            //     <Tray<RuneId> callback={remove_rune} vector={player.runes.clone()} />
            // </div>
        </>
    }
}
