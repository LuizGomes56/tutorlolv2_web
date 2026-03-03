use crate::{
    components::{
        image::{Image, ImageType},
        selector::item_filter,
    },
    utils::{EnumCast, encode_offset, hooks::use_clickout},
};
use tutorlolv2_gen::{CastId, ItemId, StatName};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ItemTrayProps {
    pub items: Vec<ItemId>,
    pub remove: Callback<usize>,
    #[prop_or_default]
    pub class: Classes,
}

#[component]
pub fn ItemTray(props: &ItemTrayProps) -> Html {
    let ItemTrayProps {
        items,
        remove,
        class,
    } = props;

    html! {
        <div class={classes!("grid", "grid-cols-7", "gap-1.5", class)}>
            {for items.iter().copied().enumerate().map(|(i, item)| {
                let onclick = {
                    let remove = remove.clone();
                    Callback::from(move |_| remove.emit(i))
                };

                html! {
                    <button
                        key={format!("tray_{i}")}
                        {onclick}
                        class={classes!(
                            "p-1", "rounded",
                            "border", "border-transparent",
                            "hover:border-red-500",
                            "bg-blue-950/75",
                            "hover:bg-red-950/75",
                            "transition-colors", "duration-200",
                            "cursor-pointer"
                        )}
                        title={format!("Remove[{i}] {name}", name = item.name())}
                        type={"button"}
                    >
                        <Image
                            src={item.image_type()}
                            class={classes!("w-7", "h-7")}
                        />
                    </button>
                }
            })}
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ItemSelectorProps {
    pub insert: Callback<ItemId>,
    pub remove: Callback<usize>,
    pub recommended: Callback<&'static [ItemId]>,
    pub items: Vec<ItemId>,
}

const ALL_STATS: [StatName; 22] = [
    StatName::AbilityHaste,
    StatName::AbilityPower,
    StatName::AdaptiveForce,
    StatName::Armor,
    StatName::ArmorPenetration,
    StatName::AttackDamage,
    StatName::AttackSpeed,
    StatName::BaseHealthRegen,
    StatName::BaseManaRegen,
    StatName::CritChance,
    StatName::CritDamage,
    StatName::GoldPer10Seconds,
    StatName::HealAndShieldPower,
    StatName::Health,
    StatName::Lethality,
    StatName::LifeSteal,
    StatName::MagicPenetration,
    StatName::MagicResist,
    StatName::Mana,
    StatName::MoveSpeed,
    StatName::Omnivamp,
    StatName::Tenacity,
];

fn contains_stat_filter(item: ItemId, stat: StatName) -> bool {
    ItemId::filter(stat).contains(&item)
}

fn item_matches_selected_stats(
    item: ItemId,
    selected_stats: &[StatName],
    strict_or_mode: bool,
) -> bool {
    if selected_stats.is_empty() {
        return true;
    }
    let mut iter = selected_stats.iter().copied();
    let action = |stat| contains_stat_filter(item, stat);
    match strict_or_mode {
        true => iter.any(action),
        false => iter.all(action),
    }
}

fn toggle_stat(list: &mut Vec<StatName>, stat: StatName) {
    match list.iter().position(|&s| s == stat) {
        Some(pos) => {
            list.remove(pos);
        }
        None => {
            list.push(stat);
        }
    }
}

#[component]
pub fn ItemSelector(props: &ItemSelectorProps) -> Html {
    let ItemSelectorProps {
        insert,
        remove,
        recommended,
        items,
    } = props;

    let is_open = use_state(|| false);
    let search = use_state(String::new);
    let selected_stats = use_state(Vec::<StatName>::new);
    let strict_or_mode = use_state(|| false);

    let dropdown_ref = use_node_ref();
    let button_ref = {
        let is_open = is_open.clone();
        use_clickout(
            Callback::from(move |_| is_open.set(false)),
            [dropdown_ref.clone()],
        )
    };

    let toggle_open = {
        let is_open = is_open.clone();
        Callback::from(move |_: MouseEvent| is_open.set(!*is_open))
    };

    let close_selector = {
        let is_open = is_open.clone();
        Callback::from(move |_: MouseEvent| is_open.set(false))
    };

    let stop_click = Callback::from(|e: MouseEvent| e.stop_propagation());

    let on_search = {
        let search = search.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            search.set(input.value());
        })
    };

    // let on_toggle_mode = {
    //     let strict_or_mode = strict_or_mode.clone();
    //     Callback::from(move |_| strict_or_mode.set(!*strict_or_mode))
    // };

    // let clear_filters = {
    //     let selected_stats = selected_stats.clone();
    //     Callback::from(move |_| selected_stats.set(Vec::new()))
    // };

    let filtered_items = {
        let query = search.trim().to_ascii_lowercase();
        let active_stats = (*selected_stats).clone();
        let strict_or = *strict_or_mode;

        ItemId::VALUES
            .iter()
            .copied()
            .filter(|&item| item_filter(item))
            .filter(|&item| item_matches_selected_stats(item, &active_stats, strict_or))
            .filter(|item| query.is_empty() || item.name().to_ascii_lowercase().contains(&query))
            .collect::<Vec<_>>()
    };

    let has_filters = !selected_stats.is_empty();

    html! {
        <>
            <button
                ref={button_ref.clone()}
                onclick={toggle_open}
                class={classes!(
                    "flex", "items-center", "gap-2",
                    "px-3", "py-1.5",
                    "rounded-md",
                    "text-sm", "font-medium",
                    "border",
                    match *is_open {
                        true => "border-amber-500 bg-amber-500/10 text-amber-300",
                        false => "border-std-700 bg-std-800 hover:bg-std-700 text-std-200"
                    },
                    "transition-colors", "duration-150"
                )}
                type={"button"}
            >
                { if *is_open { "Close" } else { "Add items" } }
                if !items.is_empty() {
                    <span class={classes!(
                        "px-1.5", "py-0.5",
                        "rounded",
                        "bg-amber-500/20", "text-amber-300",
                        "text-xs", "font-mono"
                    )}>
                        { items.len() }
                    </span>
                }
            </button>
            {
                is_open.then_some(html! {
                    <>
                        <div
                            onclick={close_selector.clone()}
                            class={classes!(
                                "fixed", "inset-0", "z-40",
                                "bg-black/50"
                            )}
                        />
                        <div class={classes!(
                            "fixed", "inset-0", "z-50", "p-4",
                            "flex", "items-center", "justify-center",
                        )}>
                            <div
                                ref={dropdown_ref.clone()}
                                onclick={stop_click}
                                class={classes!(
                                    "w-full", "max-w-7xl",
                                    "h-4/5", "rounded-md",
                                    "bg-std-950", "shadow-2xl",
                                    "shadow-black", "overflow-hidden",
                                    "flex", "flex-col"
                                )}
                            >
                                <div class={classes!(
                                    "flex", "items-center", "gap-3",
                                    "p-4", "flex-1", "bg-std-900/75"
                                )}>
                                    <span class={classes!("text-std-500", "shrink-0")}>{"⌕"}</span>
                                    <input
                                        value={(*search).clone()}
                                        oninput={on_search}
                                        placeholder={"Search items by name"}
                                        class={classes!(
                                            "flex-1", "min-w-0",
                                            "bg-transparent",
                                            "text-std-200",
                                            "placeholder:text-std-500",
                                            "outline-none"
                                        )}
                                    />
                                </div>
                                <div class={classes!("grid", "grid-cols-[auto_1fr_auto]")}>
                                    <aside class={classes!(
                                        "flex", "flex-col", "border-r",
                                        "border-std-800", "bg-std-900"
                                    )}>
                                        // <div class={classes!(
                                        //     "flex", "items-center", "justify-between",
                                        //     "px-3", "py-2.5",
                                        //     "border-b", "border-std-800"
                                        // )}>
                                        //     if has_filters {
                                        //         <button
                                        //             onclick={clear_filters}
                                        //             class={classes!(
                                        //                 "px-2", "py-0.5",
                                        //                 "text-xs", "rounded",
                                        //                 "border", "border-std-700",
                                        //                 "bg-std-800", "hover:bg-std-700",
                                        //                 "text-std-400", "hover:text-std-200",
                                        //                 "transition-colors"
                                        //             )}
                                        //             type={"button"}
                                        //         >
                                        //             {format!("Clear {n}", n = selected_stats.len())}
                                        //         </button>
                                        //     }
                                        // </div>

                                        // <div class={classes!("px-2", "pt-2", "pb-1.5")}>
                                        //     <button
                                        //         onclick={on_toggle_mode}
                                        //         class={classes!(
                                        //             "w-full",
                                        //             "flex", "items-center", "justify-between",
                                        //             "px-2.5", "py-1.5",
                                        //             "rounded-md",
                                        //             "border", "border-std-700",
                                        //             "bg-std-800", "hover:bg-std-800",
                                        //             "transition-colors", "duration-150"
                                        //         )}
                                        //         type={"button"}
                                        //     >
                                        //         <span class={classes!("text-xs", "text-std-400")}>{"Match"}</span>
                                        //         <span class={classes!(
                                        //             "px-2", "py-0.5",
                                        //             "rounded",
                                        //             "text-xs", "font-bold", "tracking-wider",
                                        //             if *strict_or_mode {
                                        //                 "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                                        //             } else {
                                        //                 "bg-std-700 text-std-300 border border-std-600"
                                        //             }
                                        //         )}>
                                        //             { if *strict_or_mode { "OR" } else { "AND" } }
                                        //         </span>
                                        //     </button>
                                        // </div>

                                        <div class={classes!(
                                            "flex", "flex-col", "flex-1",
                                            "overflow-y-auto", "py-2"
                                        )}>
                                            {for ALL_STATS.iter().copied().map(|stat| {
                                                let is_selected = selected_stats.contains(&stat);
                                                let onclick = {
                                                    let selected_stats = selected_stats.clone();
                                                    Callback::from(move |_| {
                                                        let mut next = (*selected_stats).clone();
                                                        toggle_stat(&mut next, stat);
                                                        selected_stats.set(next);
                                                    })
                                                };

                                                html! {
                                                    <button
                                                        key={stat.to_string()}
                                                        {onclick}
                                                        class={classes!(
                                                            "transition-colors",
                                                            "duration-200",
                                                            "px-4", "py-2",
                                                            match is_selected {
                                                                true => "bg-sky-950/50",
                                                                false => "bg-transparent"
                                                            }
                                                        )}
                                                        type={"button"}
                                                        title={stat.to_string()}
                                                    >
                                                        <Image
                                                            src={ImageType::StatsFilter(stat)}
                                                            class={classes!("w-4", "h-4", "shrink-0")}
                                                        />
                                                    </button>
                                                }
                                            })}
                                        </div>
                                    </aside>
                                    <section class={classes!(
                                        "flex", "overflow-auto", "flex-col",
                                        "min-w-0", "border-r", "border-std-800"
                                    )}>
                                        <div class={classes!(
                                            "flex", "items-center", "gap-2",
                                            "p-3",
                                        )}>
                                            <span class={classes!(
                                                "text-sm", "px-2", "h-7",
                                                "rounded", "content-center",
                                                "bg-blue-500/10", "border", "border-blue-500/20",
                                                "text-blue-400"
                                            )}>
                                                { format!("{n} item(s)", n = filtered_items.len()) }
                                            </span>
                                            if has_filters {
                                                <span class={classes!(
                                                    "text-sm", "px-2", "h-7",
                                                    "rounded", "content-center",
                                                    "bg-amber-500/10", "border", "border-amber-500/20",
                                                    "text-amber-400"
                                                )}>
                                                    { format!("{n} filter(s)", n = selected_stats.len()) }
                                                </span>
                                            }
                                            <button
                                                type={"button"}
                                                onclick={close_selector}
                                                class={classes!(
                                                    "text-sm", "h-7", "w-7",
                                                    "rounded", "content-center",
                                                    "bg-red-500/10", "border", "border-red-500/20",
                                                    "text-red-400"
                                                )}
                                            >
                                                {"✕"}
                                            </button>
                                        </div>
                                        <div class={classes!("flex-1", "overflow-y-auto")}>
                                            <div class={classes!("p-1.5")}>
                                                {for filtered_items.iter().copied().map(|item| {
                                                    let onclick = {
                                                        let insert = insert.clone();
                                                        Callback::from(move |_| insert.emit(item))
                                                    };

                                                    html! {
                                                        <div data_offset={encode_offset(&[item.formula()])}>
                                                            <button
                                                                key={item.name()}
                                                                {onclick}
                                                                class={classes!(
                                                                    "w-full",
                                                                    "flex", "items-center", "gap-2.5",
                                                                    "px-2.5", "py-1.5",
                                                                    "rounded-md",
                                                                    "text-left",
                                                                    "hover:bg-emerald-950/50",
                                                                    "transition-colors", "duration-200"
                                                                )}
                                                                type={"button"}
                                                            >
                                                                <div class={classes!(
                                                                    "shrink-0", "rounded"
                                                                )}>
                                                                    <Image
                                                                        src={item.image_type()}
                                                                        class={classes!("w-7", "h-7")}
                                                                    />
                                                                </div>
                                                                <span class={classes!(
                                                                    "text-std-300",
                                                                    "truncate"
                                                                )}>
                                                                    {item.name()}
                                                                </span>
                                                            </button>
                                                        </div>
                                                    }
                                                })}

                                                {filtered_items.is_empty().then_some(html! {
                                                    <div class={classes!(
                                                        "py-16", "text-center",
                                                        "text-sm", "text-std-600"
                                                    )}>
                                                        <div class={classes!("text-3xl", "mb-2")}>{"∅"}</div>
                                                        {"No items match the current filters"}
                                                    </div>
                                                })}
                                            </div>
                                        </div>
                                    </section>
                                    <aside class={classes!(
                                        "flex", "flex-col",
                                        "bg-std-900/40"
                                    )}>
                                        <div class={classes!(
                                            "flex", "items-center",
                                            "justify-between",
                                            "px-3", "py-2.5",
                                        )}>
                                            <span class={classes!(
                                                "text-xs", "font-semibold",
                                                "text-std-500"
                                            )}>{"Selected Items"}</span>
                                            if !items.is_empty() {
                                                <span class={classes!(
                                                    "px-1.5", "py-0.5",
                                                    "rounded",
                                                    "bg-amber-500/20", "text-amber-300",
                                                    "text-xs", "font-mono"
                                                )}>
                                                    { items.len() }
                                                </span>
                                            }
                                        </div>
                                        <div class={classes!("flex-1", "overflow-y-auto", "p-2")}>
                                            if items.is_empty() {
                                                <div class={classes!(
                                                    "h-full", "flex", "items-center", "justify-center",
                                                    "text-center", "text-xs", "text-std-600",
                                                    "px-4"
                                                )}>
                                                    {"Click items to add them here"}
                                                </div>
                                            } else {
                                                <ItemTray
                                                    items={items.clone()}
                                                    remove={remove.clone()}
                                                />
                                            }
                                        </div>
                                    </aside>
                                </div>
                            </div>
                        </div>
                    </>
                })
            }
        </>
    }
}
