use crate::{
    calculator::{
        Player, PlayerData,
        page::{EnemyProps, PlayerProps, TargetEntity},
        reducer::{DataAction, EnemyAction, LastAction, PlayerAction},
    },
    components::image::{Image, ImageType, Svg},
    utils::{
        encode_offset,
        hooks::{on_keydown, use_clickout},
        tray::{Tray, TrayAction, TrayEntry},
    },
};
use tutorlolv2_gen::{
    BitSetArray, CastId, ChampionId, ItemId, Position, RuneId, StatName, bitset::sizeof_bitset,
};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[hook]
pub fn use_rune_tray_callback<T: 'static>(
    props: &PlayerProps,
    callback: fn(T) -> TrayAction<Tray<RuneId>, RuneId>,
) -> Callback<T> {
    let PlayerProps {
        player,
        last_action,
    } = props.clone();

    use_callback((), move |v, _| {
        let value = callback(v);
        last_action.replace(LastAction::CurrentPlayer);
        player.dispatch(PlayerAction::Tray(value));
    })
}

#[hook]
pub fn use_item_tray_callback<T: 'static>(
    props: &PlayerProps,
    callback: fn(T) -> TrayAction<Tray<ItemId>, ItemId>,
) -> Callback<T> {
    let PlayerProps {
        player,
        last_action,
    } = props.clone();

    use_callback((), move |v, _| {
        let value = callback(v);
        last_action.replace(LastAction::CurrentPlayer);
        player.dispatch(PlayerAction::Data(DataAction::Tray(value)));
    })
}

#[hook]
pub fn use_enemy_tray_callback<T: 'static>(
    props: &EnemyProps,
    callback: fn(T) -> TrayAction<Tray<ItemId>, ItemId>,
) -> Callback<T> {
    let EnemyProps {
        enemies,
        enemy_index,
        last_action,
        ..
    } = props.clone();

    use_callback(enemy_index.clone(), move |v, enemy_index| {
        let index = **enemy_index;
        let value = callback(v);
        last_action.replace(LastAction::EnemyPlayer(index));
        enemies.dispatch(EnemyAction::Change(index, DataAction::Tray(value)));
    })
}

#[derive(PartialEq, Properties)]
pub struct RecommendedItemsProps {
    pub champion_id: ChampionId,
    pub callback: Callback<Tray<ItemId>>,
}

#[component]
pub fn RecommendedItems(props: &RecommendedItemsProps) -> Html {
    let RecommendedItemsProps {
        champion_id,
        callback,
    } = props;

    Position::ARRAY
        .into_iter()
        .map(|position| {
            let onclick = {
                let callback = callback.clone();
                let array = champion_id.recommended_items(position);
                Callback::from(move |_| callback.emit(array.iter().collect()))
            };

            html! {
                <button {onclick}>
                    <Image
                        class={classes!("w-7", "h-7")}
                        src={ImageType::Position(position)}
                    />
                </button>
            }
        })
        .collect::<Html>()
}

type FilterBitSet = BitSetArray<{ sizeof_bitset(StatName::VARIANTS) }>;

#[derive(PartialEq, Properties)]
pub struct ItemFilterProps {
    pub filters: UseStateHandle<FilterBitSet>,
}

#[component]
pub fn ItemFilter(props: &ItemFilterProps) -> Html {
    let ItemFilterProps { filters } = props;

    let toggle = use_callback(filters.clone(), move |v: StatName, filters| {
        let mut new = **filters;
        let index = v as usize;
        match new.contains(index) {
            true => new.remove(index),
            false => new.insert(index),
        };
        filters.set(new);
    });

    let clear = use_callback(filters.clone(), move |_: MouseEvent, filters| {
        filters.set(FilterBitSet::EMPTY)
    });

    let buttons = [
        StatName::AdaptiveForce,
        StatName::AttackDamage,
        StatName::CritChance,
        StatName::CritDamage,
        StatName::AttackSpeed,
        StatName::ArmorPenetration,
        StatName::Lethality,
        StatName::AbilityPower,
        StatName::Mana,
        StatName::BaseManaRegen,
        StatName::MagicPenetration,
        StatName::Health,
        StatName::BaseHealthRegen,
        StatName::HealAndShieldPower,
        StatName::Armor,
        StatName::MagicResist,
        StatName::AbilityHaste,
        StatName::MoveSpeed,
        StatName::Tenacity,
        StatName::Omnivamp,
        StatName::LifeSteal,
        StatName::GoldPer10Seconds,
    ]
    .into_iter()
    .map(|stat| {
        let onclick = {
            let toggle = toggle.clone();
            Callback::from(move |_| toggle.emit(stat))
        };

        let contains = filters.contains(stat as usize);

        html! {
            <button
                {onclick}
                title={stat.to_string()}
                class={classes!(
                    "border-r-3", "py-1.5", "px-4",
                    match contains {
                        true => classes!("border-r-cyan-400", "bright"),
                        false => classes!("border-r-transparent")
                    },
                )}
            >
                <Image
                    class={classes!("w-4", "h-4", match contains {
                        true => "contrast-100",
                        false => "contrast-0"
                    })}
                    src={ImageType::StatsFilter(stat)}
                />
            </button>
        }
    })
    .collect::<Html>();

    let clear_button = html! {
        <button
            onclick={clear}
            title={"Clear"}
            class={classes!("p-1")}
        >
            <span class={classes!("w-4", "h-4")}>
                {"x"}
            </span>
        </button>
    };

    html! {
        <div class={classes!("flex", "flex-col")}>
            // {clear_button}
            {buttons}
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ItemSelectorProps {
    pub player_props: PlayerProps,
    pub enemy_props: EnemyProps,
    pub entity: UseStateHandle<TargetEntity>,
    pub is_open: UseStateHandle<bool>,
}

#[component]
pub fn ItemSelector(props: &ItemSelectorProps) -> Html {
    let ItemSelectorProps {
        player_props,
        enemy_props,
        entity,
        is_open,
    } = props;

    let query = use_state(String::new);
    let filters = use_state_eq(|| FilterBitSet::EMPTY);

    let dropdown_ref = {
        let is_open = is_open.clone();
        use_clickout(Callback::from(move |_| is_open.set(false)), [])
    };

    let oninput = {
        let query = query.clone();
        use_callback((), move |e: InputEvent, _| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();
            query.set(value);
        })
    };

    {
        let is_open = is_open.clone();
        use_effect_with((), move |_| on_keydown(27, move || is_open.set(false)));
    }

    fn make_row(
        entity: &UseStateHandle<TargetEntity>,
        target_entity: TargetEntity,
        champion_id: ChampionId,
        items: &Tray<ItemId>,
        remove: Callback<u64>,
        recommendations: Callback<Tray<ItemId>>,
    ) -> Html {
        let remover = items
            .iter()
            .map(|&entry| {
                let TrayEntry { id, value } = entry;

                let onclick = {
                    let remove = remove.clone();
                    Callback::from(move |_| remove.emit(id))
                };

                let data_offset = encode_offset(&[value.formula()]);

                html! {
                    <button
                        {onclick}
                        {data_offset}
                        key={id}
                        data_classes={classes!("-translate-x-[calc(100%-36px)]")}
                    >
                        <Image
                            class={classes!(
                                "w-9", "h-9", "border-2",
                                "border-std-700"
                            )}
                            src={ImageType::from(value)}
                        />
                    </button>
                }
            })
            .collect::<Html>();

        let onclick = {
            let entity = entity.clone();
            Callback::from(move |_| {
                entity.set(target_entity);
            })
        };

        let data_offset = encode_offset(&[champion_id.formula()]);

        html! {
            <div class={classes!(
                "flex", "flex-col", "gap-4", "p-2",
                if **entity == target_entity { "bg-std-800" } else { "bg-transparent" }
            )}>
                <div class={classes!("grid", "grid-cols-6", "gap-2")}>
                    <button
                        {onclick}
                        {data_offset}
                        data_classes={classes!("-translate-x-[calc(100%-36px)]")}
                    >
                        <Image
                            class={classes!("w-7", "h-7")}
                            src={ImageType::Champion(champion_id)}
                        />
                    </button>
                    <RecommendedItems
                        {champion_id}
                        callback={recommendations}
                    />
                </div>
                <div class={classes!("grid", "grid-cols-6", "gap-2")}>
                    {remover}
                </div>
            </div>
        }
    }

    let tray = {
        let player_tray = {
            let Player {
                data:
                    PlayerData {
                        ref items,
                        champion_id,
                        ..
                    },
                ..
            } = *player_props.player;

            let player_remove = {
                let player_props = player_props.clone();

                Callback::from(move |id| {
                    player_props.last_action.replace(LastAction::CurrentPlayer);
                    player_props
                        .player
                        .dispatch(PlayerAction::Data(DataAction::Tray(
                            TrayAction::RemoveById(id),
                        )));
                })
            };

            let player_replace = {
                let player_props = player_props.clone();

                Callback::from(move |v: Tray<ItemId>| {
                    player_props.last_action.replace(LastAction::CurrentPlayer);
                    player_props
                        .player
                        .dispatch(PlayerAction::Data(DataAction::Tray(TrayAction::Replace(v))));
                })
            };

            make_row(
                entity,
                TargetEntity::Player,
                champion_id,
                items,
                player_remove,
                player_replace,
            )
        };

        let enemy_tray = enemy_props
            .enemies
            .iter()
            .enumerate()
            .map(|(i, enemy)| {
                let PlayerData {
                    ref items,
                    champion_id,
                    ..
                } = *enemy.as_ref();

                let enemy_remove = {
                    let enemy_props = enemy_props.clone();

                    Callback::from(move |id: u64| {
                        enemy_props.last_action.replace(LastAction::EnemyPlayer(i));
                        enemy_props.enemies.dispatch(EnemyAction::Change(
                            i,
                            DataAction::Tray(TrayAction::RemoveById(id)),
                        ));
                    })
                };

                let enemy_replace = {
                    let enemy_props = enemy_props.clone();

                    Callback::from(move |v: Tray<ItemId>| {
                        enemy_props.last_action.replace(LastAction::EnemyPlayer(i));
                        enemy_props.enemies.dispatch(EnemyAction::Change(
                            i,
                            DataAction::Tray(TrayAction::Replace(v)),
                        ));
                    })
                };

                html! {
                    <div key={format!("enemy_row_{i}")}>
                        {make_row(
                            entity,
                            TargetEntity::Enemy(i),
                            champion_id,
                            items,
                            enemy_remove,
                            enemy_replace,
                        )}
                    </div>
                }
            })
            .collect::<Html>();

        html! {
            <div class={classes!("flex", "flex-col", "gap-4", "overflow-auto")}>
                {player_tray}
                {enemy_tray}
            </div>
        }
    };

    let insert = match **entity {
        TargetEntity::Player => {
            let player_props = player_props.clone();

            Callback::from(move |item| {
                player_props.last_action.replace(LastAction::CurrentPlayer);
                player_props
                    .player
                    .dispatch(PlayerAction::Data(DataAction::Tray(TrayAction::Insert(
                        item,
                    ))));
            })
        }
        TargetEntity::Enemy(i) => {
            let enemy_props = enemy_props.clone();

            Callback::from(move |item| {
                enemy_props.last_action.replace(LastAction::EnemyPlayer(i));
                enemy_props.enemies.dispatch(EnemyAction::Change(
                    i,
                    DataAction::Tray(TrayAction::Insert(item)),
                ));
            })
        }
    };

    let options = ItemId::VALUES
        .iter()
        .copied()
        .filter(|item| {
            query.is_empty()
                || item
                    .name()
                    .to_ascii_lowercase()
                    .contains(query.to_ascii_lowercase().as_str())
        })
        .filter(|item| {
            filters.is_empty()
                || filters
                    .into_iter()
                    .all(|v| ItemId::filter(StatName::from_u8_unchecked(v as _)).contains(item))
        })
        .enumerate()
        .map(|(i, item)| {
            let insert = insert.clone();
            let onclick = Callback::from(move |_| insert.emit(item));

            let data_offset = encode_offset(&[item.formula()]);

            html! {
                <button
                    {onclick}
                    {data_offset}
                    data_classes={(i % 17 > 17 / 2).then_some("-translate-x-[calc(100%-36px)]")}
                    class={classes!("flex", "flex-col", "gap-1", "w-fit")}
                >
                    <Image
                        src={ImageType::from(item)}
                        class={classes!("w-9", "h-9", "border-2", "border-std-700")}
                    />
                    <span class={classes!("text-std-500", "font-medium", "text-center")}>
                        {item.price()}
                    </span>
                </button>
            }
        })
        .collect::<Html>();

    html! {
        <div class={classes!(
            if **is_open { "flex" } else { "hidden" },
            "fixed", "inset-0", "z-50",
            "items-center", "justify-center",
            "bg-black/70",
        )}>
            <div
                ref={dropdown_ref}
                class={classes!(
                    "grid", "grid-cols-[1fr_auto]",
                    "bg-std-950", "w-full", "max-w-7xl",
                    "h-[80vh]", "overflow-hidden"
                )}
            >
                <div class={classes!(
                    "flex", "flex-col", "gap-4",
                    "w-full", "p-4",
                    "h-full", "min-h-0"
                )}>
                    <ItemSearch {oninput} />
                    <div class={classes!(
                        "grid", "grid-cols-[auto_1fr]",
                        "gap-4", "flex-1", "min-h-0"
                    )}>
                        <ItemFilter {filters} />
                        <div class={classes!(
                            "flex", "flex-wrap", "gap-4",
                            "content-start",
                            "min-h-0", "h-full",
                            "overflow-y-auto"
                        )}>
                            {options}
                        </div>
                    </div>
                </div>
                {tray}
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ItemSearchProps {
    pub oninput: Callback<InputEvent>,
}

#[component]
pub fn ItemSearch(props: &ItemSearchProps) -> Html {
    let ItemSearchProps { oninput } = props;
    html! {
        <div
            class={classes!(
                "group",
                "flex", "items-center", "gap-2",
                "border", "border-transparent",
                "focus-within:border-std-800",
                "px-4"
            )}
        >
            <Svg
                class={classes!(
                    "w-4", "h-4", "shrink-0",
                    "text-std-300",
                    "group-focus-within:text-white"
                )}
                src={"/svgs/search.svg"}
            />
            <input
                class={classes!(
                    "placeholder:text-std-500",
                    "bg-transparent",
                    "text-sm", "p-3", "outline-none"
                )}
                placeholder={"Click Here to Search"}
                {oninput}
            />
        </div>
    }
}
