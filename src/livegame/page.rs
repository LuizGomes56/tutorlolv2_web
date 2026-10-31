use crate::{
    components::{
        image::{Image, ImageType},
        selector::Selector,
        stack::{Stack, StackInsert, StackRemover, StackTable},
        tables::{body::DamageRenderer, empty::EmptyTable, header::TableHeader},
    },
    hover::{self, Docs},
    livegame::{
        CurrentPlayer, Enemy, Game, ability_levels::AbilityLevelsDisplay, banner::Banner,
        dragon::DragonDisplay, scoreboard::ScoreboardDisplay,
    },
    utils::{Loading, glue::get_desktop_data, use_setter},
};
use std::{cell::Cell, rc::Rc, time::Duration};
use strum::IntoEnumIterator;
use tutorlolv2::{Ctx, CtxVar, ItemId, TypeMetadata, bitset::BitSet, model::Damages};
use yew::{
    platform::{spawn_local, time::sleep},
    prelude::*,
};

// Keep the fixed candidate pair in the data, but show its column only when at
// least one enemy has damage there. Header and every row share this decision.
fn simulated_items_meta<'a>(
    owned: &Rc<[TypeMetadata<ItemId>]>,
    candidate: ItemId,
    damages: impl IntoIterator<Item = &'a Damages>,
) -> Rc<[TypeMetadata<ItemId>]> {
    let offset = owned.len() * 2;
    let visible = damages.into_iter().any(|damage| {
        debug_assert_eq!(damage.items.len(), offset + 2);
        damage.items[offset] != 0 || damage.items[offset + 1] != 0
    });

    if !visible {
        return owned.clone();
    }

    owned
        .iter()
        .copied()
        .chain(core::iter::once(candidate.data().metadata))
        .collect()
}

#[component]
pub fn Livegame() -> Html {
    let game_data = use_state(|| Err::<Game, Box<dyn core::error::Error>>(Loading.into()));

    {
        let game_data = game_data.clone();

        use_effect_with((), |_| {
            let active = Rc::new(Cell::new(true));
            let running = active.clone();

            spawn_local(async move {
                while running.get() {
                    let data = get_desktop_data().await;

                    if !running.get() {
                        break;
                    }

                    game_data.set(data);

                    sleep(Duration::from_secs(1)).await;
                }
            });
            move || active.set(false)
        });
    };

    #[cfg(feature = "tauri")]
    let sharing = html!(<crate::desktop::sharing::SharingBar />);

    #[cfg(not(feature = "tauri"))]
    let sharing = Html::default();

    html! {
        <>
            { sharing }
            <LivegameView
                data={(*game_data).as_ref().ok().map(|g| Rc::new(g.clone()))}
                error={(*game_data).as_ref().err().map(ToString::to_string).unwrap_or_default()}
            />
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct LivegameViewProps {
    pub data: Option<Rc<Game>>,
    #[prop_or_default]
    pub error: String,
}

#[component]
pub fn LivegameView(props: &LivegameViewProps) -> Html {
    let stack = use_reducer(Stack::default);
    let stack_push = Stack::use_push(&stack);
    let siml_item = use_state(|| const { ItemId::SIML[0] });
    let siml_item_callback = use_setter(&siml_item);

    match props.data.as_deref() {
        Some(data) => {
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

            let player_idents = {
                let CurrentPlayer {
                    base_stats,
                    bonus_stats,
                    current_stats,
                    level,
                    ..
                } = current_player;

                let ctx = Ctx {
                    base_armor: base_stats.armor,
                    base_ad: base_stats.attack_damage,
                    base_attack_speed: base_stats.attack_speed,
                    base_magic_resist: base_stats.magic_resist,
                    base_health: base_stats.max_health,
                    base_mana: base_stats.max_mana,

                    bonus_armor: bonus_stats.armor,
                    bonus_ad: bonus_stats.attack_damage,
                    bonus_attack_speed: bonus_stats.attack_speed,
                    bonus_magic_resist: bonus_stats.magic_resist,
                    bonus_health: bonus_stats.max_health,
                    bonus_mana: bonus_stats.max_mana,

                    ability_power: current_stats.ability_power,
                    armor: current_stats.armor,
                    armor_penetration_flat: current_stats.armor_penetration_flat,
                    armor_penetration_percent: current_stats.armor_penetration_percent,
                    attack_damage: current_stats.attack_damage,
                    attack_speed: current_stats.attack_speed,
                    crit_chance: current_stats.crit_chance,
                    crit_damage: current_stats.crit_damage,
                    current_health: current_stats.current_health,
                    magic_penetration_flat: current_stats.magic_penetration_flat,
                    magic_penetration_percent: current_stats.magic_penetration_percent,
                    magic_resist: current_stats.magic_resist,
                    max_health: current_stats.max_health,
                    max_mana: current_stats.max_mana,
                    current_mana: current_stats.current_mana,
                    level: level.get() as _,

                    ..Default::default()
                };

                let idents = CtxVar::iter()
                    .filter(|var| hover::ident_display_value(*var, ctx[*var]) != 0.0)
                    .collect::<Box<[_]>>();

                hover::encode_idents(&idents, &ctx)
            };

            let champion_id = current_player.champion_id;
            let siml_index = ItemId::SIML
                .iter()
                .position(|&v| v == *siml_item)
                .unwrap_or(0);

            let siml_items_meta = simulated_items_meta(
                items_meta,
                *siml_item,
                enemies.iter().map(|enemy| &enemy.siml_items[siml_index]),
            );

            let get_damages = |enemy_id, damages, ctx| {
                html! {
                    <tr>
                        <td
                            data_idents={hover::encode_idents(&CtxVar::ENEMY_VARS, ctx)}
                            data_docs={Docs::global(enemy_id).encode()}
                            class={classes!("w-12")}
                        >
                            <Image src={ImageType::from(enemy_id)} />
                        </td>
                        { damages }
                    </tr>
                }
            };

            let damages = enemies
                .iter()
                .map(|enemy| {
                    get_damages(
                        enemy.champion_id,
                        DamageRenderer::new(
                            &enemy.damages,
                            champion_id,
                            current_player.adaptive_type,
                            items_meta,
                            runes_meta,
                            None,
                        ),
                        &enemy.damages.ctx,
                    )
                })
                .collect::<Html>();

            let siml_damages = |i: usize| {
                enemies
                    .iter()
                    .map(|enemy| {
                        get_damages(
                            enemy.champion_id,
                            DamageRenderer::new(
                                &enemy.siml_items[i],
                                champion_id,
                                current_player.adaptive_type,
                                &siml_items_meta,
                                runes_meta,
                                Some(&enemy.damages),
                            ),
                            &enemy.damages.ctx,
                        )
                    })
                    .collect::<Html>()
            };

            let enemy_rows = enemies
                .iter()
                .map(|enemy| (enemy.champion_id, enemy.total_damage(), enemy.item_scores()))
                .collect::<Vec<_>>();

            let mut seen = BitSet::EMPTY;

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
                    html! {
                        <th data_docs={Docs::global(item_id).encode()}>
                            <Image src={ImageType::from(item_id)} />
                        </th>
                    }
                })
                .collect::<Html>();

            let recm_body = enemy_rows
                .iter()
                .map(|(enemy_id, base, list)| {
                    html! {
                        <tr>
                            <td
                                data_docs={Docs::global(*enemy_id).encode()}
                                class={classes!("w-10")}
                            >
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
                                            for dmg in damage {
                                                <span class={classes!("text-sm")}>
                                                    {dmg}
                                                </span>
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
                            { recm_header }
                        </tr>
                    </thead>
                    <tbody>{ recm_body }</tbody>
                </table>
            };

            html! {
                <div class={classes!(
                    "flex", "flex-col", "mb-96",
                    "overflow-hidden", "flex-1",
                    "xl:flex-row"
                )}>
                    <div class={classes!(
                        "flex", "py-4", "pl-4", "pr-2",
                        "flex-col", "gap-4", "min-w-80"
                    )}>
                        <Banner
                            data_idents={player_idents}
                            riot_id={&current_player.riot_id}
                            {game_time}
                            {champion_id}
                        />
                        <ScoreboardDisplay
                            ally_team={current_player.team}
                            {scoreboard}
                        />
                        <div class={classes!("grid", "grid-cols-[auto_auto]", "gap-4")}>
                            <DragonDisplay {dragons} />
                            <AbilityLevelsDisplay {champion_id} {ability_levels} />
                        </div>
                    </div>
                    <div class={classes!(
                        "flex", "flex-col", "py-4", "pl-2",
                        "pr-4", "gap-4", "flex-1", "overflow-auto"
                    )}>
                        <div class={classes!("box", "overflow-auto")}>
                            <table class={classes!("data-table")}>
                                <TableHeader {champion_id} {items_meta} {runes_meta} />
                                <tbody>{ damages }</tbody>
                            </table>
                        </div>
                        <div class={classes!("box", "overflow-auto")}>
                            <div class={classes!(
                                "grid", "grid-cols-1",
                                "items-start", "gap-4",
                                "2xl:grid-cols-3",
                            )}>
                                <StackInsert
                                    callback={stack_push}
                                    {items_meta}
                                    {runes_meta}
                                    {champion_id}
                                />
                                <StackRemover
                                    stack={&stack}
                                    {champion_id}
                                    {items_meta}
                                    {runes_meta}
                                />
                                <div class={classes!("overflow-auto")}>
                                    <StackTable<Enemy>
                                        {enemies}
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
                                { "Bonus damage after buying new item" }
                            </div>
                            { recm_table }
                        </div>
                        <div class={classes!("box", "overflow-auto")}>
                            <div class={classes!("h-fit", "p-4")}>
                                <Selector<ItemId>
                                    value={*siml_item}
                                    array={&ItemId::SIML as &'static [_]}
                                    callback={siml_item_callback}
                                    img_class={classes!("w-10", "h-10")}
                                    input_class={classes!("text-xl", "font-medium")}
                                />
                            </div>
                            <table class={classes!("data-table")}>
                                <TableHeader
                                    {champion_id}
                                    items_meta={&siml_items_meta}
                                    {runes_meta}
                                />
                                <tbody>{ siml_damages(siml_index) }</tbody>
                            </table>
                        </div>
                    </div>
                </div>
            }
        }
        None => {
            html! {
                <div class={classes!(
                    "flex", "gap-4", "p-4",
                    "overflow-hidden", "flex-1"
                )}>
                    if !props.error.is_empty() {
                        <p role={"status"}>{ &props.error }</p>
                    }
                    <EmptyTable
                        rows={5}
                    />
                    <div class={classes!("box", "h-96")} />
                </div>
            }
        }
    }
}

#[cfg(test)]
mod purchase_tests {
    use super::*;

    #[test]
    fn selected_purchase_appends_metadata_after_owned_columns() {
        let owned: Rc<[_]> = vec![ItemId::NashorsTooth.data().metadata].into();
        let damage = Damages {
            items: vec![10, 20, 7, 0].into_boxed_slice(),
            ..Default::default()
        };
        let selected = simulated_items_meta(&owned, ItemId::BladeOfTheRuinedKing, [&damage]);
        assert_eq!(
            selected.iter().map(|m| m.kind).collect::<Vec<_>>(),
            vec![ItemId::NashorsTooth, ItemId::BladeOfTheRuinedKing]
        );
        assert_eq!(owned.len(), 1);
        let empty: Rc<[_]> = Rc::from([]);
        let damage = Damages {
            items: vec![0, 7].into_boxed_slice(),
            ..Default::default()
        };
        assert_eq!(
            simulated_items_meta(&empty, ItemId::NashorsTooth, [&damage])[0].kind,
            ItemId::NashorsTooth
        );
    }

    #[test]
    fn all_zero_candidate_column_is_hidden_despite_owned_damage() {
        let owned: Rc<[_]> = vec![ItemId::NashorsTooth.data().metadata].into();
        let damage = Damages {
            items: vec![50, 60, 0, 0].into_boxed_slice(),
            ..Default::default()
        };
        for candidate in [ItemId::NashorsTooth, ItemId::SapphireCrystal] {
            let selected = simulated_items_meta(&owned, candidate, [&damage, &damage]);
            assert!(Rc::ptr_eq(&selected, &owned));
        }
        assert!(Rc::ptr_eq(
            &simulated_items_meta(&owned, ItemId::NashorsTooth, []),
            &owned
        ));
    }

    #[test]
    fn any_nonzero_enemy_keeps_candidate_column_for_every_row() {
        let owned: Rc<[_]> = vec![ItemId::NashorsTooth.data().metadata].into();
        let zero = Damages {
            items: vec![50, 60, 0, 0].into_boxed_slice(),
            ..Default::default()
        };
        for pair in [[0, 12], [15, 0]] {
            let nonzero = Damages {
                items: vec![50, 60, pair[0], pair[1]].into_boxed_slice(),
                ..Default::default()
            };
            for rows in [[&zero, &nonzero], [&nonzero, &zero]] {
                let selected = simulated_items_meta(&owned, ItemId::BladeOfTheRuinedKing, rows);
                assert_eq!(selected.len(), 2);
                assert_eq!(selected[1].kind, ItemId::BladeOfTheRuinedKing);
            }
        }
    }
}
