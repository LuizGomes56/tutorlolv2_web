use crate::{
    calculator::{
        components::inputs::{
            abilities::Abilities,
            banner::Banner,
            checkbox::Checkbox,
            dragon::{DragonInput, use_dragons},
            exceptions::{ChampionExceptionSelector, ExceptionSelector},
            stats::{StatCell, Stats},
        },
        page::{PlayerProps, TargetEntity},
        reducer::{DataAction, LastAction, PlayerAction},
    },
    components::{
        image::{DragonImage, ImageType},
        selector::SelectorButton,
    },
    model::{Dragons, DragonsAction, PlayerStats},
    utils::Print,
};
use tutorlolv2_gen::{ChampionId, ItemId, RuneId};
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
    pub dragons: UseReducerHandle<Dragons>,
}

#[component]
pub fn PlayerInput(props: &PlayerInputProps) -> Html {
    let PlayerInputProps {
        player_props,
        open_item_menu,
        dragons,
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

    let item_exception_callback = use_data_callback(player_props, DataAction::ModifyItemExc);
    let rune_exception_callback = use_player_callback(player_props, PlayerAction::ModifyRuneExc);
    let stack_callback = use_data_callback(player_props, DataAction::Stacks);
    let infer_stats_callback = use_data_callback(player_props, DataAction::InferStats);
    let is_mega_gnar_callback = use_data_callback(player_props, DataAction::IsMegaGnar);

    let ally_fire = use_dragons(dragons, &player_props.last_action, DragonsAction::AllyFire);
    let ally_earth = use_dragons(dragons, &player_props.last_action, DragonsAction::AllyEarth);

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
                <div class={classes!("flex", "flex-col", "gap-2", "my-2", "w-full")}>
                    <SelectorButton
                        title={"Items"}
                        onclick={{
                            let open_item_menu = open_item_menu.clone();
                            Callback::from(move |_| {
                                open_item_menu.emit(TargetEntity::Player);
                            })
                        }}
                        length={player.data.items.len()}
                    />
                    <SelectorButton
                        title={"Runes"}
                        onclick={{
                            Callback::from(move |_| {})
                            // let open_item_menu = open_item_menu.clone();
                            // Callback::from(move |_| {
                            //     open_item_menu.emit(TargetEntity::Player);
                            // })
                        }}
                        length={player.runes.len()}
                    />
                </div>
                <div class={classes!(
                    "grid", "grid-cols-[auto_1fr_1fr]",
                    "gap-x-2", "px-4", "py-3", "gap-y-1.5",
                    "empty:hidden"
                )}>
                    if data.infer_stats {
                        <DragonInput
                            title={"Fire dragons"}
                            oninput={ally_fire}
                            src={DragonImage::Fire}
                            value={dragons.ally_fire}
                        />
                        <DragonInput
                            title={"Earth dragons"}
                            oninput={ally_earth}
                            src={DragonImage::Earth}
                            value={dragons.ally_earth}
                        />
                    }
                    <ChampionExceptionSelector
                        champion_id={player.data.champion_id}
                        stacks={player.data.stacks}
                        callback={stack_callback}
                        ally={true}
                    />
                    <ExceptionSelector<{ ItemId::SIZE_OF_EXCEPTIONS }, ItemId>
                        values={player.data.items.clone()}
                        exceptions={player.data.item_exceptions.clone()}
                        callback={item_exception_callback}
                        filter={ItemId::exceptions(true)}
                    />
                    <ExceptionSelector<{ RuneId::SIZE_OF_EXCEPTIONS }, RuneId>
                        values={player.runes.clone()}
                        exceptions={player.rune_exceptions.clone()}
                        callback={rune_exception_callback}
                        filter={RuneId::exceptions()}
                    />
                </div>
                if player.data.champion_id == ChampionId::Gnar {
                    <Checkbox
                        checked={player.data.is_mega_gnar}
                        callback={is_mega_gnar_callback}
                        label={"Mega Gnar"}
                    />
                }
                <Checkbox
                    checked={data.infer_stats}
                    callback={infer_stats_callback}
                    label={"Infer Stats"}
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
                                let number = value.parse().unwrap_or(1).max(1);
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
