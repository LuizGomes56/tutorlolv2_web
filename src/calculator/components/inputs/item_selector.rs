use crate::{
    calculator::{
        components::inputs::{enemies::use_enemy_data_callback, player::use_data_callback},
        page::{EnemyProps, PlayerProps, TargetEntity},
        reducer::DataAction,
    },
    components::image::{Image, ImageType},
    utils::{encode_offset, hooks::use_clickout},
};
use tutorlolv2_gen::{CastId, ItemId, Position, StatName};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ItemSelectorProps {
    pub player_props: PlayerProps,
    pub enemy_props: EnemyProps,
    pub entity: UseStateHandle<Option<TargetEntity>>,
}

#[component]
pub fn ItemSelector(props: &ItemSelectorProps) -> Html {
    let ItemSelectorProps {
        player_props,
        enemy_props,
        entity,
    } = props;

    let player_insert_callback = use_data_callback(player_props, DataAction::InsertItem);
    let player_remove_callback = use_data_callback(player_props, DataAction::RemoveItem);
    let player_recommendations = use_data_callback(player_props, DataAction::SetItemVec);

    let enemy_insert_callback = use_enemy_data_callback(enemy_props, DataAction::InsertItem);
    let enemy_remove_callback = use_enemy_data_callback(enemy_props, DataAction::RemoveItem);
    let enemy_recommendations = use_enemy_data_callback(enemy_props, DataAction::SetItemVec);

    let filters = use_state(Vec::<StatName>::new);

    let set_player = {
        let entity = entity.clone();
        use_callback((), move |_: MouseEvent, _| {
            entity.set(Some(TargetEntity::Player));
        })
    };

    let set_enemy = {
        let entity = entity.clone();
        use_callback((), move |index: usize, _| {
            entity.set(Some(TargetEntity::Enemy(index)));
        })
    };

    let items_ref = ItemId::VALUES
        .iter()
        .copied()
        .filter(|item| {
            if filters.is_empty() {
                return true;
            }
            filters
                .iter()
                .all(|&stat| ItemId::filter(stat).contains(item))
        })
        // .filter(|&item| item_filter(item))
        // .filter(|&item| item_matches_selected_stats(item, &active_stats, strict_or))
        // .filter(|item| query.is_empty() || item.name().to_ascii_lowercase().contains(&query))
        .collect::<Vec<_>>();

    let filter_box = use_memo(filters, |filters| {
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
        ]
        .into_iter()
        .map(|stat| {
            let contains = filters.contains(&stat);
            let onclick = {
                let filters = filters.clone();
                Callback::from(move |_| {
                    let mut new = (*filters).clone();
                    match contains {
                        true => new.retain(|&f| f != stat),
                        false => new.push(stat),
                    }
                    filters.set(new);
                })
            };
            html! {
                <button
                    class={classes!(
                        "w-full", "px-3", "py-2",
                        "flex", "items-center", "gap-2",
                        "rounded-lg", "border",
                        "text-left", "text-sm",
                        "transition-colors", "duration-150",
                        match contains {
                            true => "border-sky-500/40 bg-sky-500/12 text-sky-100",
                            false => "border-transparent bg-transparent text-std-300 hover:bg-std-900/80 hover:text-white",
                        }
                    )}
                    {onclick}
                >
                    <Image
                        class={classes!("w-4", "h-4", "shrink-0")}
                        src={ImageType::StatsFilter(stat)}
                    />
                    <span class={classes!("truncate")}>{stat.name()}</span>
                </button>
            }
        })
        .collect::<Html>();

        html! {
            <div class={classes!("flex", "flex-col", "gap-1.5", "p-2")}>
                {buttons}
            </div>
        }
    });

    // let search = use_state(String::new);

    let dropdown_ref = {
        let entity = entity.clone();
        use_clickout(Callback::from(move |_| entity.set(None)), [])
    };

    let player = &player_props.player;
    let enemies = &enemy_props.enemies;

    let Some(current_entity) = &**entity else {
        return Default::default();
    };

    let (insert, remove, recommendations) = match current_entity {
        TargetEntity::Player => (
            player_insert_callback,
            player_remove_callback,
            player_recommendations,
        ),
        TargetEntity::Enemy(_) => (
            enemy_insert_callback,
            enemy_remove_callback,
            enemy_recommendations,
        ),
    };

    let (champion_id, items) = match current_entity {
        TargetEntity::Player => {
            let data = &player.data;
            (data.champion_id, &data.items)
        }
        TargetEntity::Enemy(index) => {
            let data = &enemies[*index];
            (data.champion_id, &data.items)
        }
    };

    let tray = {
        let buttons = items
            .iter()
            .enumerate()
            .map(|(i, &item_id)| {
                let data_offset = encode_offset(&[item_id.formula()]);
                html! {
                    <span {data_offset}>
                        <button
                            class={classes!(
                                "flex", "items-center", "justify-center",
                                "rounded-lg",
                                "p-1.5",
                                "border", "border-std-700/70",
                                "bg-std-900/70",
                                "hover:border-red-400",
                                "hover:bg-red-500/10",
                                "transition-colors", "duration-200",
                                "cursor-pointer"
                            )}
                            onclick={{
                                let remove = remove.clone();
                                Callback::from(move |_| {
                                    remove.emit(i);
                                })
                            }}
                        >
                            <Image
                                class={classes!("w-7", "h-7")}
                                src={ImageType::from(item_id)}
                            />
                        </button>
                    </span>
                }
            })
            .collect::<Html>();

        html! {
            <div class={classes!("grid", "grid-cols-3", "gap-2", "p-3")}>
                {buttons}
            </div>
        }
    };

    let positions = {
        let buttons = Position::ARRAY
            .iter()
            .map(|&position| {
                html! {
                    <button
                        class={classes!(
                            "p-2",
                            "rounded-lg",
                            "border", "border-std-700/70",
                            "bg-std-900/70",
                            "hover:bg-amber-500/10",
                            "hover:border-amber-400/60",
                            "transition-colors", "duration-150"
                        )}
                        onclick={{
                            let recommendations = recommendations.clone();
                            Callback::from(move |_| {
                                recommendations.emit(champion_id.recommended_items(position));
                            })
                        }}
                    >
                        <Image
                            class={classes!("w-6", "h-6")}
                            src={ImageType::Position(position)}
                        />
                    </button>
                }
            })
            .collect::<Html>();

        html! {
            <div class={classes!("grid", "grid-cols-5", "gap-2")}>
                {buttons}
            </div>
        }
    };

    let options = {
        let buttons = items_ref
            .iter()
            .map(|&item_id| {
                let data_offset = encode_offset(&[item_id.formula()]);
                html! {
                    <span {data_offset}>
                        <button
                            class={classes!(
                                "w-full",
                                "flex", "items-center", "gap-3",
                                "px-3", "py-2",
                                "rounded-lg",
                                "text-left",
                                "border", "border-transparent",
                                "hover:border-emerald-500/20",
                                "hover:bg-emerald-500/8",
                                "transition-colors", "duration-150"
                            )}
                            onclick={{
                                let insert = insert.clone();
                                Callback::from(move |_| {
                                    insert.emit(item_id);
                                })
                            }}
                        >
                            <Image
                                class={classes!("w-7", "h-7", "shrink-0")}
                                src={ImageType::from(item_id)}
                            />
                            <span class={classes!("truncate", "text-std-200")}>{item_id.name()}</span>
                        </button>
                    </span>
                }
            })
            .collect::<Html>();

        html! {
            <div class={classes!("grid", "grid-cols-2", "gap-1.5", "p-3")}>
                {buttons}
            </div>
        }
    };

    let entity_selector = html! {
        <div class={classes!("flex", "items-center", "gap-2")}>
            <button
                class={classes!(
                    "p-1", "rounded-lg", "border", "transition-colors", "duration-150",
                    match current_entity {
                        TargetEntity::Player => "border-emerald-400 bg-emerald-500/10",
                        _ => "border-transparent hover:border-std-600 hover:bg-std-900/70",
                    }
                )}
                onclick={set_player}
            >
                <Image
                    class={classes!("w-7", "h-7")}
                    src={ImageType::from(player.data.champion_id)}
                />
            </button>
            for (i, enemy) in enemies.iter().enumerate() {
                <button
                    class={classes!(
                        "p-1", "rounded-lg", "border", "transition-colors", "duration-150",
                        match current_entity {
                            TargetEntity::Enemy(j) if j == &i => "border-orange-400 bg-orange-500/10",
                            _ => "border-transparent hover:border-std-600 hover:bg-std-900/70",
                        }
                    )}
                    onclick={{
                        let set_enemy = set_enemy.clone();
                        Callback::from(move |_| {
                            set_enemy.emit(i);
                        })
                    }}
                >
                    <Image
                        class={classes!("w-7", "h-7")}
                        src={ImageType::from(enemy.champion_id)}
                    />
                </button>
            }
        </div>
    };

    html! {
        <div class={classes!(
            "fixed", "inset-0", "z-50",
            "flex", "items-center", "justify-center",
            "bg-black/60", "p-3", "sm:p-4",
        )}>
            <div
                ref={dropdown_ref}
                class={classes!(
                    "w-full", "max-w-7xl",
                    "h-[88vh]",
                    "rounded-xl",
                    "border", "border-std-800",
                    "bg-std-950",
                    "shadow-2xl", "shadow-black/80",
                    "overflow-hidden",
                    "flex", "flex-col"
                )}
            >
                <div class={classes!(
                    "grid", "grid-cols-[1fr_auto_auto]",
                    "items-center", "gap-3",
                    "p-4",
                    "border-b", "border-std-800",
                    "bg-std-900/50"
                )}>
                    <input
                        type={"search"}
                        placeholder={"Search items by name"}
                        class={classes!(
                            "w-full",
                            "px-3", "py-2.5",
                            "rounded-lg",
                            "border", "border-std-700",
                            "bg-std-900/80",
                            "text-std-100",
                            "placeholder:text-std-500",
                            "outline-none"
                        )}
                    />
                    {entity_selector}
                    {positions}
                </div>

                <div class={classes!(
                    "grid", "grid-cols-[250px_1fr_220px]",
                    "flex-1", "min-h-0"
                )}>
                    <div class={classes!(
                        "overflow-y-auto",
                        "border-r", "border-std-800",
                        "bg-std-900/35"
                    )}>
                        {(*filter_box).clone()}
                    </div>

                    <div class={classes!(
                        "overflow-y-auto",
                        "bg-std-950"
                    )}>
                        {options}
                    </div>

                    <div class={classes!(
                        "overflow-y-auto",
                        "border-l", "border-std-800",
                        "bg-std-900/35"
                    )}>
                        {tray}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ItemButtonProps {
    pub onclick: Callback<MouseEvent>,
    pub length: usize,
}

#[component]
pub fn ItemButton(props: &ItemButtonProps) -> Html {
    let ItemButtonProps {
        ref onclick,
        length,
    } = *props;
    html! {
        <button
            class={classes!(
                "flex", "items-center", "justify-between", "gap-2",
                "px-4", "py-2.5",
                "rounded-lg",
                "border", "border-transparent",
                "text-std-200",
                "hover:bg-amber-400/10",
                "hover:border-amber-500/20",
                "transition-colors", "duration-200"
            )}
            {onclick}
        >
            <span>{"Items"}</span>
            <span class={classes!(
                "px-2", "py-1", "rounded-md",
                "bg-amber-500/20", "text-amber-300",
                "text-sm", "font-mono"
            )}>
                {length}
            </span>
        </button>
    }
}
