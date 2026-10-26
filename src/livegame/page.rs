use crate::{
    components::{
        errorlog::errorlog,
        image::{Image, ImageType},
        selector::Selector,
        stack::{Stack, StackInsert, StackRemover, StackTable},
        tables::{body::DamageRenderer, empty::EmptyTable, header::TableHeader},
    },
    hover::{self, Docs},
    livegame::{
        Enemy, Game, ability_levels::AbilityLevelsDisplay, banner::Banner, dragon::DragonDisplay,
        scoreboard::ScoreboardDisplay,
    },
    utils::{Loading, glue::get_desktop_data, use_setter},
};
use std::{rc::Rc, time::Duration};
use tutorlolv2::{ChampionId, CtxVar, ItemId, TypeMetadata, bitset::BitSet, model::Damages};
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
    let game_data = use_state(|| Err::<Game, _>(Loading.into()));
    let stack = use_reducer(Stack::default);
    let stack_push = Stack::use_push(&stack);
    let siml_item = use_state(|| ItemId::SIML[0]);
    let siml_item_callback = use_setter(&siml_item);

    {
        let game_data = game_data.clone();

        use_effect_with((), |_| {
            spawn_local(async move {
                loop {
                    let data = get_desktop_data().await;

                    game_data.set(data);

                    sleep(Duration::from_secs(1)).await;
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
            let siml_index = ItemId::SIML
                .iter()
                .position(|&v| v == *siml_item)
                .unwrap_or(0);

            let siml_items_meta = simulated_items_meta(
                items_meta,
                *siml_item,
                enemies.iter().map(|enemy| &enemy.siml_items[siml_index]),
            );

            let get_damages = |enemy_id: ChampionId, damages, data_idents: Option<String>| {
                html! {
                    <tr>
                        <td
                            {data_idents}
                            data_docs={Docs::global(enemy_id).encode()}
                            class={classes!("w-12")}
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
                        DamageRenderer::new(
                            &enemy.damages,
                            champion_id,
                            items_meta,
                            runes_meta,
                            None,
                        ),
                        hover::encode_idents(&CtxVar::ENEMY_VARS, &enemy.damages.ctx),
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
                                &siml_items_meta,
                                runes_meta,
                                Some(&enemy.damages),
                            ),
                            hover::encode_idents(&CtxVar::ENEMY_VARS, &enemy.damages.ctx),
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
                            <td data_docs={Docs::global(*enemy_id).encode()} class={classes!("w-10")}>
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
                            {recm_header}
                        </tr>
                    </thead>
                    <tbody>{recm_body}</tbody>
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
                            <AbilityLevelsDisplay
                                {champion_id}
                                {ability_levels}
                            />
                        </div>
                    </div>
                    <div class={classes!(
                        "flex", "flex-col", "py-4", "pl-2",
                        "pr-4", "gap-4", "flex-1", "overflow-auto"
                    )}>
                        <div class={classes!("box", "overflow-auto")}>
                            <table class={classes!("data-table")}>
                                <TableHeader
                                    {champion_id}
                                    {items_meta}
                                    {runes_meta}
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
                                {"Bonus damage after buying new item"}
                            </div>
                            {recm_table}
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
                                <tbody>
                                    {siml_damages(siml_index)}
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
