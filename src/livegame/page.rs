use crate::{
    components::{
        errorlog::errorlog,
        image::{Image, ImageType},
        selector::Selector,
        stack::{Stack, StackInsert, StackRemover, StackTable},
        tables::{empty::EmptyTable, header::TableHeader},
    },
    livegame::{
        Enemy, Game, ability_levels::AbilityLevelsDisplay, banner::Banner, dragon::DragonDisplay,
        scoreboard::ScoreboardDisplay,
    },
    utils::{Fetch, Loading, encode_offset, glue::get_data, use_setter},
};
use tutorlolv2_gen::{CastId, ChampionId, ItemId, ItemsBitSet, SIMULATED_ITEMS_ENUM};
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

#[component]
pub fn Livegame() -> Html {
    let game_data = use_state(|| Err::<Game, _>(Loading.into()));
    let stack = use_reducer(Stack::default);
    let stack_push = Stack::use_push(&stack);
    let siml_item = use_state(|| SIMULATED_ITEMS_ENUM[0]);
    let siml_item_callback = use_setter(&siml_item);

    {
        let game_data = game_data.clone();
        use_effect_with((), |_| {
            spawn_local(async move {
                loop {
                    let data = get_data().await;
                    game_data.set(data);
                    sleep(Fetch::REFRESH_RATE).await;
                }
            });
        });
    };

    match &*game_data {
        Ok(data) => {
            let Game {
                ref current_player,
                ref enemies,
                ref scoreboard,
                ref items_meta,
                ref runes_meta,
                game_time,
                ability_levels,
                dragons,
            } = *data;

            let champion_id = current_player.champion_id;

            let get_damages = |enemy_id: ChampionId, damages| {
                html! {
                    <tr>
                        <td
                            class={classes!("w-12")}
                            data_offset={encode_offset(&[enemy_id.formula()])}
                        >
                            <Image src={ImageType::from(enemy_id)} />
                        </td>
                        {damages}
                    </tr>
                }
            };

            let damages = enemies
                .iter()
                .map(|enemy| {
                    get_damages(
                        enemy.champion_id,
                        enemy
                            .damages
                            .to_html(champion_id, items_meta, runes_meta, None),
                    )
                })
                .collect::<Html>();

            let siml_damages = |i: usize| {
                enemies
                    .iter()
                    .map(|enemy| {
                        get_damages(
                            enemy.champion_id,
                            enemy.siml_items[i].to_html(
                                champion_id,
                                items_meta,
                                runes_meta,
                                Some(&enemy.damages),
                            ),
                        )
                    })
                    .collect::<Html>()
            };

            let enemy_rows = enemies
                .iter()
                .map(|enemy| {
                    (
                        enemy.champion_id,
                        enemy.total_damage(),
                        enemy.item_scores(champion_id),
                    )
                })
                .collect::<Vec<_>>();

            let mut seen = ItemsBitSet::EMPTY;
            let columns = enemy_rows
                .iter()
                .flat_map(|(.., list)| list.iter())
                .filter_map(|&(_, item_id)| {
                    seen.insert_const(item_id.index() as _).then_some(item_id)
                })
                .collect::<Vec<_>>();

            let recm_header = columns
                .iter()
                .copied()
                .map(|item_id| {
                    let data_offset = encode_offset(core::array::from_ref(&item_id.formula()));

                    html! {
                        <th {data_offset}>
                            <Image src={ImageType::from(item_id)} />
                        </th>
                    }
                })
                .collect::<Html>();

            let recm_body = enemy_rows
                .iter()
                .map(|(enemy_id, base, list)| {
                    let data_offset = encode_offset(core::array::from_ref(&enemy_id.formula()));

                    html! {
                        <tr>
                            <td {data_offset} class={classes!("w-10")}>
                                <Image src={ImageType::from(*enemy_id)} />
                            </td>
                            {
                                for columns.iter().copied().map(|item_id| {
                                    let damage = list
                                        .iter()
                                        .find(|(_, id)| *id == item_id)
                                        .map(|(damage, _)| *damage - *base);

                                    html! {
                                        <td>
                                            {
                                                damage.map(|damage| html! {
                                                    <span class={classes!("text-sm")}>
                                                        {damage}
                                                    </span>
                                                }).unwrap_or_default()
                                            }
                                        </td>
                                    }
                                })
                            }
                        </tr>
                    }
                })
                .collect::<Html>();

            let recm_table = html! {
                <table class={classes!("data-table")}>
                    <thead>
                        <tr>
                            <th />
                            {recm_header}
                        </tr>
                    </thead>
                    <tbody>{recm_body}</tbody>
                </table>
            };

            html! {
                <div class={classes!(
                    "flex", "flex-col", "gap-4", "mb-96",
                    "p-4", "overflow-hidden", "flex-1",
                    "xl:flex-row"
                )}>
                    <div class={classes!("flex", "flex-col", "gap-4", "min-w-80")}>
                        <Banner
                            riot_id={current_player.riot_id.clone()}
                            {game_time}
                            {champion_id}
                        />
                        <ScoreboardDisplay
                            ally_team={current_player.team}
                            scoreboard={scoreboard.clone()}
                        />
                        <div class={classes!("grid", "grid-cols-[auto_auto]", "gap-4")}>
                            <DragonDisplay {dragons} />
                            <AbilityLevelsDisplay
                                {champion_id}
                                {ability_levels}
                            />
                        </div>
                    </div>
                    <div class={classes!("flex", "flex-col", "gap-4", "flex-1")}>
                        <div class={classes!("box", "overflow-auto")}>
                            <table class={classes!("data-table")}>
                                <TableHeader
                                    {champion_id}
                                    items_meta={items_meta.clone()}
                                    runes_meta={runes_meta.clone()}
                                />
                                <tbody>{damages}</tbody>
                            </table>
                        </div>
                        <div class={classes!("box", "overflow-auto")}>
                            <div class={classes!(
                                "grid", "grid-cols-1",
                                "items-start", "gap-4",
                                "2xl:grid-cols-3",
                            )}>
                                <StackInsert
                                    callback={stack_push.clone()}
                                    items_meta={items_meta.clone()}
                                    runes_meta={runes_meta.clone()}
                                    {champion_id}
                                />
                                <StackRemover
                                    stack={stack.clone()}
                                    {champion_id}
                                    items_meta={items_meta.clone()}
                                    runes_meta={runes_meta.clone()}
                                />
                                <div class={classes!("overflow-auto")}>
                                    <StackTable<Enemy>
                                        enemies={enemies.clone()}
                                        stack={stack.reconcile(champion_id, items_meta, runes_meta)}
                                        level={current_player.level}
                                    />
                                </div>
                            </div>
                        </div>
                        <div class={classes!("box", "overflow-auto")}>
                            <div class={classes!(
                                "text-xl", "font-medium", "text-std-400",
                                "px-6", "py-5"
                            )}>
                                {"Bonus damage after buying new item"}
                            </div>
                            {recm_table}
                        </div>
                        <div class={classes!("box", "overflow-auto")}>
                            <div class={classes!("h-fit", "p-4")}>
                                <Selector<ItemId>
                                    value={*siml_item}
                                    array={&SIMULATED_ITEMS_ENUM as &'static [_]}
                                    callback={siml_item_callback}
                                    img_class={classes!("w-10", "h-10")}
                                    input_class={classes!("text-xl", "font-medium")}
                                />
                            </div>
                            <table class={classes!("data-table")}>
                                <TableHeader
                                    {champion_id}
                                    items_meta={items_meta.clone()}
                                    runes_meta={runes_meta.clone()}
                                />
                                <tbody>
                                    {siml_damages(SIMULATED_ITEMS_ENUM
                                        .iter()
                                        .position(|&v| v == *siml_item)
                                        .unwrap_or(0)
                                    )}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            }
        }
        Err(e) => {
            html! {
                <div class={classes!(
                    "flex", "gap-4", "p-4",
                    "overflow-hidden", "flex-1"
                )}>
                    {errorlog(e)}
                    <EmptyTable rows={5} />
                    <div class={classes!("box", "h-96")} />
                </div>
            }
        }
    }
}
