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
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlInputElement};
use yew::prelude::*;

const FILTER_STATS: [StatName; 21] = [
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
];

#[hook]
pub fn use_event_callback<IN, OUT, F>(f: F) -> Callback<IN, OUT>
where
    IN: 'static,
    OUT: 'static,
    F: Fn(IN) -> OUT + 'static,
{
    let latest = use_mut_ref(|| None::<Callback<IN, OUT>>);
    *latest.borrow_mut() = Some(Callback::from(f));

    let latest_for_cb = latest.clone();
    use_callback((), move |input: IN, _| {
        let borrowed = latest_for_cb.borrow();
        let callback = borrowed.as_ref().expect("callback must be initialized");
        callback.emit(input)
    })
}

fn event_attr(event: &MouseEvent, attr: &str) -> Option<String> {
    let mut current = event.target()?.dyn_into::<Element>().ok();

    while let Some(element) = current {
        if let Some(value) = element.get_attribute(attr) {
            return Some(value);
        }
        current = element.parent_element();
    }

    None
}

fn event_attr_index(event: &MouseEvent, attr: &str) -> Option<usize> {
    event_attr(event, attr)?.parse::<usize>().ok()
}

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

    let query = use_state_eq(String::new);
    let filters = use_state_eq(Vec::<StatName>::new);

    let player_insert_callback = use_data_callback(player_props, DataAction::InsertItem);
    let player_remove_callback = use_data_callback(player_props, DataAction::RemoveItem);
    let player_recommendations = use_data_callback(player_props, DataAction::SetItemVec);

    let enemy_insert_callback = use_enemy_data_callback(enemy_props, DataAction::InsertItem);
    let enemy_remove_callback = use_enemy_data_callback(enemy_props, DataAction::RemoveItem);
    let enemy_recommendations = use_enemy_data_callback(enemy_props, DataAction::SetItemVec);

    let dropdown_ref = {
        let entity = entity.clone();
        use_clickout(Callback::from(move |_| entity.set(None)), [])
    };

    let current_entity = (*entity).clone();

    let player_champion_id = player_props.player.data.champion_id;
    let player_items = player_props.player.data.items.clone();

    let enemy_champion_ids = enemy_props
        .enemies
        .iter()
        .map(|enemy| enemy.champion_id)
        .collect::<Vec<_>>();

    let enemy_items = enemy_props
        .enemies
        .iter()
        .map(|enemy| enemy.items.clone())
        .collect::<Vec<_>>();

    let owned_items = match current_entity.as_ref() {
        Some(TargetEntity::Player) => player_items.clone(),
        Some(TargetEntity::Enemy(i)) => enemy_items.get(*i).cloned().unwrap_or_default(),
        None => Vec::new(),
    };

    let normalized_query = use_memo((*query).clone(), |query| query.trim().to_ascii_lowercase());

    let filtered_items = {
        let filters_snapshot = (*filters).clone();
        let query_snapshot = (*normalized_query).clone();

        use_memo((filters_snapshot, query_snapshot), |(filters, query)| {
            ItemId::VALUES
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
                .filter(|item| query.is_empty() || item.name().to_ascii_lowercase().contains(query))
                .collect::<Vec<_>>()
        })
    };

    let on_search = {
        let query = query.clone();
        Callback::from(move |event: InputEvent| {
            let value = event.target_unchecked_into::<HtmlInputElement>().value();
            query.set(value);
        })
    };

    let on_toggle_filter = {
        let filters = filters.clone();
        use_event_callback(move |event: MouseEvent| {
            let Some(index) = event_attr_index(&event, "data-stat-index") else {
                return;
            };

            let Some(&stat) = FILTER_STATS.get(index) else {
                return;
            };

            let mut next = (*filters).clone();
            if let Some(pos) = next.iter().position(|&it| it == stat) {
                next.remove(pos);
            } else {
                next.push(stat);
            }
            filters.set(next);
        })
    };

    let on_select_entity = {
        let entity = entity.clone();
        let current_entity = current_entity.clone();

        use_event_callback(move |event: MouseEvent| {
            if event_attr(&event, "data-target-player").is_some() {
                if *current_entity != Some(TargetEntity::Player) {
                    entity.set(Some(TargetEntity::Player));
                }
                return;
            }

            let Some(index) = event_attr_index(&event, "data-target-enemy-index") else {
                return;
            };

            let next = Some(TargetEntity::Enemy(index));
            if *current_entity != next {
                entity.set(next);
            }
        })
    };

    let on_apply_recommendation = {
        let current_entity = current_entity.clone();
        let player_recommendations = player_recommendations.clone();
        let enemy_recommendations = enemy_recommendations.clone();
        let enemy_champion_ids = enemy_champion_ids.clone();

        use_event_callback(move |event: MouseEvent| {
            let Some(index) = event_attr_index(&event, "data-position-index") else {
                return;
            };

            let Some(&position) = Position::ARRAY.get(index) else {
                return;
            };

            match current_entity.as_ref() {
                Some(TargetEntity::Player) => {
                    player_recommendations.emit(player_champion_id.recommended_items(position));
                }
                Some(TargetEntity::Enemy(i)) => {
                    if let Some(&champion_id) = enemy_champion_ids.get(*i) {
                        enemy_recommendations.emit(champion_id.recommended_items(position));
                    }
                }
                None => {}
            }
        })
    };

    let on_add_item = {
        let current_entity = current_entity.clone();
        let filtered_items = (*filtered_items).clone();
        let player_insert_callback = player_insert_callback.clone();
        let enemy_insert_callback = enemy_insert_callback.clone();

        use_event_callback(move |event: MouseEvent| {
            let Some(index) = event_attr_index(&event, "data-item-index") else {
                return;
            };

            let Some(&item_id) = filtered_items.get(index) else {
                return;
            };

            match current_entity.as_ref() {
                Some(TargetEntity::Player) => player_insert_callback.emit(item_id),
                Some(TargetEntity::Enemy(_)) => enemy_insert_callback.emit(item_id),
                None => {}
            }
        })
    };

    let on_remove_item = {
        let current_entity = current_entity.clone();
        let player_remove_callback = player_remove_callback.clone();
        let enemy_remove_callback = enemy_remove_callback.clone();

        use_event_callback(move |event: MouseEvent| {
            let Some(index) = event_attr_index(&event, "data-remove-index") else {
                return;
            };

            match current_entity.as_ref() {
                Some(TargetEntity::Player) => player_remove_callback.emit(index),
                Some(TargetEntity::Enemy(_)) => enemy_remove_callback.emit(index),
                None => {}
            }
        })
    };

    let filter_box = {
        let active_filters = (*filters).clone();
        let on_toggle_filter = on_toggle_filter.clone();

        use_memo(
            (active_filters, on_toggle_filter),
            |(filters, on_toggle_filter)| {
                html! {
                    <div
                        class={classes!("flex", "flex-col", "gap-1", "p-2")}
                        onclick={on_toggle_filter.clone()}
                    >
                        {for FILTER_STATS.iter().enumerate().map(|(index, &stat)| {
                            let selected = filters.contains(&stat);

                            html! {
                                <button
                                    type={"button"}
                                    data-stat-index={index.to_string()}
                                    class={classes!(
                                        "w-full",
                                        "flex", "items-center", "gap-2.5",
                                        "px-2.5", "py-2",
                                        "rounded-lg",
                                        "border",
                                        "text-left", "text-[13px]",
                                        "transition-all", "duration-150",
                                        if selected {
                                            "border-sky-500/25 bg-sky-500/10 text-sky-100"
                                        } else {
                                            "border-transparent bg-transparent text-std-300 hover:bg-std-900/90 hover:text-white"
                                        }
                                    )}
                                >
                                    <Image
                                        class={classes!("w-4", "h-4", "shrink-0")}
                                        src={ImageType::StatsFilter(stat)}
                                    />
                                    <span class={classes!("truncate")}>{stat.name()}</span>
                                </button>
                            }
                        })}
                    </div>
                }
            },
        )
    };

    let positions = {
        let on_apply_recommendation = on_apply_recommendation.clone();

        use_memo(on_apply_recommendation, |on_apply_recommendation| {
            html! {
                <div
                    class={classes!("grid", "grid-cols-5", "gap-1.5")}
                    onclick={on_apply_recommendation.clone()}
                >
                    {for Position::ARRAY.iter().enumerate().map(|(index, &position)| {
                        html! {
                            <button
                                type={"button"}
                                data-position-index={index.to_string()}
                                class={classes!(
                                    "flex", "items-center", "justify-center",
                                    "rounded-lg",
                                    "p-2",
                                    "border", "border-std-800",
                                    "bg-std-900/80",
                                    "hover:bg-amber-500/8",
                                    "hover:border-amber-500/30",
                                    "transition-all", "duration-150"
                                )}
                            >
                                <Image
                                    class={classes!("w-5", "h-5")}
                                    src={ImageType::Position(position)}
                                />
                            </button>
                        }
                    })}
                </div>
            }
        })
    };

    let options = {
        let filtered_items = (*filtered_items).clone();
        let on_add_item = on_add_item.clone();

        use_memo(
            (filtered_items, on_add_item),
            |(filtered_items, on_add_item)| {
                if filtered_items.is_empty() {
                    return html! {
                        <div class={classes!(
                            "h-full",
                            "flex", "items-center", "justify-center",
                            "p-6"
                        )}>
                            <div class={classes!(
                                "w-full", "max-w-sm",
                                "rounded-xl",
                                "border", "border-dashed", "border-std-800",
                                "bg-[#0b0d12]",
                                "px-5", "py-8",
                                "text-center"
                            )}>
                                <div class={classes!("text-sm", "font-medium", "text-std-200")}>
                                    {"No items found"}
                                </div>
                                <div class={classes!("mt-1", "text-[12px]", "text-std-500")}>
                                    {"Try changing the search term or removing some filters."}
                                </div>
                            </div>
                        </div>
                    };
                }

                html! {
                    <div
                        class={classes!("grid", "grid-cols-1", "2xl:grid-cols-2", "gap-1", "p-2")}
                        onclick={on_add_item.clone()}
                    >
                        {for filtered_items.iter().enumerate().map(|(index, &item_id)| {
                            let data_offset = encode_offset(&[item_id.formula()]);
                            html! {
                                <span {data_offset}>
                                    <button
                                        type={"button"}
                                        data-item-index={index.to_string()}
                                        class={classes!(
                                            "w-full",
                                            "flex", "items-center", "gap-3",
                                            "px-2.5", "py-2.5",
                                            "rounded-lg",
                                            "text-left",
                                            "border", "border-transparent",
                                            "bg-transparent",
                                            "hover:border-emerald-500/12",
                                            "hover:bg-emerald-500/6",
                                            "transition-all", "duration-150"
                                        )}
                                    >
                                        <Image
                                            class={classes!("w-7", "h-7", "shrink-0")}
                                            src={ImageType::from(item_id)}
                                        />
                                        <span class={classes!("truncate", "text-std-100", "text-[13px]")}>
                                            {item_id.name()}
                                        </span>
                                    </button>
                                </span>
                            }
                        })}
                    </div>
                }
            },
        )
    };

    let tray = {
        let owned_items = owned_items.clone();
        let on_remove_item = on_remove_item.clone();

        use_memo(
            (owned_items, on_remove_item),
            |(owned_items, on_remove_item)| {
                if owned_items.is_empty() {
                    return html! {
                        <div class={classes!(
                            "h-full",
                            "flex", "items-center", "justify-center",
                            "p-3"
                        )}>
                            <div class={classes!(
                                "w-full",
                                "rounded-xl",
                                "border", "border-dashed", "border-std-800",
                                "bg-[#0b0d12]",
                                "px-4", "py-6",
                                "text-center", "text-[12px]", "text-std-500"
                            )}>
                                {"No items selected for this entity."}
                            </div>
                        </div>
                    };
                }

                html! {
                    <div
                        class={classes!("grid", "grid-cols-3", "gap-1.5", "p-3")}
                        onclick={on_remove_item.clone()}
                    >
                        {for owned_items.iter().enumerate().map(|(index, &item_id)| {
                            let data_offset = encode_offset(&[item_id.formula()]);
                            html! {
                                <span {data_offset}>
                                    <button
                                        type={"button"}
                                        data-remove-index={index.to_string()}
                                        class={classes!(
                                            "flex", "items-center", "justify-center",
                                            "rounded-lg",
                                            "p-1.5",
                                            "border", "border-std-800",
                                            "bg-std-900/80",
                                            "hover:border-red-500/35",
                                            "hover:bg-red-500/8",
                                            "transition-all", "duration-150",
                                            "cursor-pointer"
                                        )}
                                    >
                                        <Image
                                            class={classes!("w-7", "h-7")}
                                            src={ImageType::from(item_id)}
                                        />
                                    </button>
                                </span>
                            }
                        })}
                    </div>
                }
            },
        )
    };

    let entity_selector = {
        let current_entity = current_entity.clone();
        let on_select_entity = on_select_entity.clone();
        let player_champion_id = player_props.player.data.champion_id;
        let enemy_champion_ids = enemy_champion_ids.clone();

        use_memo(
            (
                current_entity.clone(),
                on_select_entity,
                player_champion_id,
                enemy_champion_ids,
            ),
            |(current_entity, on_select_entity, player_champion_id, enemy_champion_ids)| {
                let data_offset = encode_offset(&[player_champion_id.formula()]);
                html! {
                    <div
                        class={classes!("flex", "items-center", "gap-1.5", "flex-wrap")}
                        onclick={on_select_entity.clone()}
                    >
                        <button
                            type={"button"}
                            {data_offset}
                            data-target-player={"1"}
                            class={classes!(
                                "p-1",
                                "rounded-lg",
                                "border",
                                "transition-all", "duration-150",
                                if matches!(**current_entity, Some(TargetEntity::Player)) {
                                    "border-emerald-500/35 bg-emerald-500/10"
                                } else {
                                    "border-transparent hover:border-std-700 hover:bg-std-900/80"
                                }
                            )}
                        >
                            <Image
                                class={classes!("w-7", "h-7")}
                                src={ImageType::from(*player_champion_id)}
                            />
                        </button>

                        {for enemy_champion_ids.iter().enumerate().map(|(index, &champion_id)| {
                            let active = matches!(**current_entity, Some(TargetEntity::Enemy(i)) if i == index);
                            let data_offset = encode_offset(&[champion_id.formula()]);
                            html! {
                                <button
                                    type={"button"}
                                    data-target-enemy-index={index.to_string()}
                                    {data_offset}
                                    class={classes!(
                                        "p-1",
                                        "rounded-lg",
                                        "border",
                                        "transition-all", "duration-150",
                                        if active {
                                            "border-orange-500/35 bg-orange-500/10"
                                        } else {
                                            "border-transparent hover:border-std-700 hover:bg-std-900/80"
                                        }
                                    )}
                                >
                                    <Image
                                        class={classes!("w-7", "h-7")}
                                        src={ImageType::from(champion_id)}
                                    />
                                </button>
                            }
                        })}
                    </div>
                }
            },
        )
    };

    let current_entity_label = match current_entity.as_ref() {
        Some(TargetEntity::Player) => "Player",
        Some(TargetEntity::Enemy(_)) => "Enemy",
        None => "None",
    };

    if current_entity.is_none() {
        return Html::default();
    }

    html! {
        <div class={classes!(
            "fixed", "inset-0", "z-50",
            "flex", "items-center", "justify-center",
            "bg-black/70",
            "p-3"
        )}>
            <div
                ref={dropdown_ref}
                class={classes!(
                    "w-full", "max-w-[1280px]",
                    "h-[88vh]",
                    "rounded-2xl",
                    "border", "border-std-800",
                    "bg-[#0b0d12]",
                    "overflow-hidden",
                    "flex", "flex-col"
                )}
            >
                <div class={classes!(
                    "grid", "grid-cols-[minmax(0,1fr)_auto_auto]",
                    "items-center", "gap-3",
                    "px-4", "py-3",
                    "border-b", "border-std-800",
                    "bg-[#0f1218]"
                )}>
                    <div class={classes!("min-w-0", "space-y-2")}>
                        <div class={classes!(
                            "flex", "items-center", "gap-2",
                            "text-[11px]", "uppercase", "tracking-[0.18em]",
                            "text-std-500"
                        )}>
                            <span>{"Item Manager"}</span>
                            <span class={classes!("h-px", "flex-1", "bg-std-800")} />
                        </div>

                        <div class={classes!(
                            "flex", "items-center", "gap-2",
                            "min-w-0",
                            "px-3", "py-2.5",
                            "rounded-xl",
                            "border", "border-std-800",
                            "bg-[#0c1016]"
                        )}>
                            <span class={classes!("text-std-500", "text-sm", "leading-none")}>{"⌕"}</span>
                            <input
                                type={"search"}
                                value={(*query).clone()}
                                oninput={on_search}
                                placeholder={"Search item by name..."}
                                class={classes!(
                                    "w-full",
                                    "bg-transparent",
                                    "text-[13px]", "text-std-100",
                                    "placeholder:text-std-500",
                                    "outline-none",
                                    "border-0", "ring-0",
                                )}
                            />
                            <span class={classes!(
                                "px-1.5", "py-0.5",
                                "rounded-md",
                                "bg-emerald-500/10",
                                "text-emerald-300",
                                "text-[11px]", "font-medium"
                            )}>
                                {filtered_items.len()}
                            </span>
                        </div>
                    </div>

                    <div class={classes!(
                        "flex", "flex-col", "gap-1.5",
                        "px-3", "py-2.5",
                        "rounded-xl",
                        "border", "border-std-800",
                        "bg-[#0c1016]",
                        "min-w-[170px]"
                    )}>
                        <div class={classes!(
                            "flex", "items-center", "justify-between", "gap-2",
                            "text-[10px]", "uppercase", "tracking-[0.16em]", "text-std-500"
                        )}>
                            <span>{"Target"}</span>
                            <span class={classes!(
                                "px-1.5", "py-0.5",
                                "rounded-md",
                                "bg-amber-500/10",
                                "text-amber-300",
                                "text-[10px]", "font-medium"
                            )}>
                                {current_entity_label}
                            </span>
                        </div>
                        {(*entity_selector).clone()}
                    </div>

                    <div class={classes!(
                        "flex", "flex-col", "gap-1.5",
                        "px-3", "py-2.5",
                        "rounded-xl",
                        "border", "border-std-800",
                        "bg-[#0c1016]",
                        "min-w-[210px]"
                    )}>
                        <div class={classes!(
                            "flex", "items-center", "justify-between", "gap-2"
                        )}>
                            <span class={classes!(
                                "text-[10px]", "uppercase", "tracking-[0.16em]", "text-std-500"
                            )}>
                                {"Recommended"}
                            </span>
                            <span class={classes!(
                                "px-1.5", "py-0.5",
                                "rounded-md",
                                "bg-amber-500/10",
                                "text-amber-300",
                                "text-[10px]", "font-medium"
                            )}>
                                {"Role"}
                            </span>
                        </div>
                        {(*positions).clone()}
                    </div>
                </div>

                <div class={classes!(
                    "grid", "grid-cols-[240px_minmax(0,1fr)_220px]",
                    "flex-1", "min-h-0"
                )}>
                    <aside class={classes!(
                        "flex", "flex-col",
                        "min-h-0",
                        "border-r", "border-std-800",
                        "bg-[#0d1016]"
                    )}>
                        <div class={classes!(
                            "flex", "items-center", "justify-between",
                            "px-3", "py-2.5",
                            "border-b", "border-std-800"
                        )}>
                            <div class={classes!("space-y-0.5")}>
                                <div class={classes!("text-[12px]", "font-medium", "text-std-100")}>
                                    {"Filters"}
                                </div>
                                <div class={classes!("text-[11px]", "text-std-500")}>
                                    {"AND matching"}
                                </div>
                            </div>
                            <span class={classes!(
                                "px-1.5", "py-0.5",
                                "rounded-md",
                                "bg-sky-500/10",
                                "text-sky-300",
                                "text-[11px]", "font-medium"
                            )}>
                                {filters.len()}
                            </span>
                        </div>

                        <div class={classes!("flex-1", "overflow-y-auto")}>
                            {(*filter_box).clone()}
                        </div>
                    </aside>

                    <section class={classes!(
                        "flex", "flex-col",
                        "min-h-0",
                        "bg-[#0b0d12]"
                    )}>
                        <div class={classes!(
                            "flex", "items-center", "justify-between", "gap-3",
                            "px-3.5", "py-2.5",
                            "border-b", "border-std-800",
                            "bg-[#0d1016]"
                        )}>
                            <div class={classes!("min-w-0", "space-y-0.5")}>
                                <div class={classes!("text-[12px]", "font-medium", "text-std-100")}>
                                    {"Catalog"}
                                </div>
                                <div class={classes!("text-[11px]", "text-std-500")}>
                                    {"Click an item to add it"}
                                </div>
                            </div>

                            <div class={classes!("flex", "items-center", "gap-1.5", "shrink-0")}>
                                if !query.is_empty() {
                                    <span class={classes!(
                                        "px-1.5", "py-0.5",
                                        "rounded-md",
                                        "bg-emerald-500/10",
                                        "text-emerald-300",
                                        "text-[10px]", "font-medium"
                                    )}>
                                        {"Search"}
                                    </span>
                                }
                                if !filters.is_empty() {
                                    <span class={classes!(
                                        "px-1.5", "py-0.5",
                                        "rounded-md",
                                        "bg-sky-500/10",
                                        "text-sky-300",
                                        "text-[10px]", "font-medium"
                                    )}>
                                        {format!("{} filter(s)", filters.len())}
                                    </span>
                                }
                                <span class={classes!(
                                    "px-1.5", "py-0.5",
                                    "rounded-md",
                                    "bg-blue-500/10",
                                    "text-blue-300",
                                    "text-[10px]", "font-medium"
                                )}>
                                    {format!("{} result(s)", filtered_items.len())}
                                </span>
                            </div>
                        </div>

                        <div class={classes!("flex-1", "overflow-y-auto")}>
                            {(*options).clone()}
                        </div>
                    </section>

                    <aside class={classes!(
                        "flex", "flex-col",
                        "min-h-0",
                        "border-l", "border-std-800",
                        "bg-[#0d1016]"
                    )}>
                        <div class={classes!(
                            "flex", "items-center", "justify-between",
                            "px-3", "py-2.5",
                            "border-b", "border-std-800"
                        )}>
                            <div class={classes!("space-y-0.5")}>
                                <div class={classes!("text-[12px]", "font-medium", "text-std-100")}>
                                    {"Tray"}
                                </div>
                                <div class={classes!("text-[11px]", "text-std-500")}>
                                    {"Current entity items"}
                                </div>
                            </div>
                            <span class={classes!(
                                "px-1.5", "py-0.5",
                                "rounded-md",
                                "bg-amber-500/10",
                                "text-amber-300",
                                "text-[11px]", "font-medium"
                            )}>
                                {owned_items.len()}
                            </span>
                        </div>

                        <div class={classes!("flex-1", "overflow-y-auto")}>
                            {(*tray).clone()}
                        </div>
                    </aside>
                </div>
            </div>
        </div>
    }
}
