use crate::{
    calculator::components::inputs::selector::item_filter,
    components::image::{Image, ImageType},
    utils::{EnumCast, hooks::use_mouseout},
};
use tutorlolv2_gen::{ItemId, StatName};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ItemTrayProps {
    pub items: Box<[ItemId]>,
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
        <div class={classes!("flex", "flex-wrap", "gap-1.5", class.clone())}>
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
                            "group", "relative",
                            "p-1", "rounded",
                            "border", "border-zinc-700/60",
                            "bg-zinc-800/80",
                            "hover:border-amber-500/60",
                            "hover:bg-zinc-700/80",
                            "transition-colors", "duration-150",
                            "cursor-pointer"
                        )}
                        title={format!("Remove {name}", name = item.name())}
                        type={"button"}
                    >
                        <Image
                            src={item.image_type()}
                            class={classes!("w-7", "h-7", "block")}
                        />
                        <span class={classes!(
                            "absolute", "-top-1", "-right-1",
                            "w-3.5", "h-3.5",
                            "rounded-full",
                            "bg-zinc-900", "border", "border-zinc-600",
                            "text-[8px]", "text-zinc-400",
                            "flex", "items-center", "justify-center",
                            "opacity-0", "group-hover:opacity-100",
                            "transition-opacity"
                        )}>{"×"}</span>
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
    pub items: Box<[ItemId]>,
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
    StatName::CriticalStrikeChance,
    StatName::CriticalStrikeDamage,
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
        recommended: _recommended,
        items,
    } = props;

    let is_open = use_state(|| false);
    let search = use_state(String::new);
    let selected_stats = use_state(Vec::<StatName>::new);
    let strict_or_mode = use_state(|| false);

    let dropdown_ref = use_node_ref();
    let button_ref = {
        let is_open = is_open.clone();
        use_mouseout(
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

    let on_toggle_mode = {
        let strict_or_mode = strict_or_mode.clone();
        Callback::from(move |_| strict_or_mode.set(!*strict_or_mode))
    };

    let clear_filters = {
        let selected_stats = selected_stats.clone();
        Callback::from(move |_| selected_stats.set(Vec::new()))
    };

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
        <div class={classes!("space-y-2")}>
            // — Tray + trigger row —
            <div class={classes!("flex", "items-center", "gap-3", "flex-wrap")}>
                <button
                    ref={button_ref.clone()}
                    onclick={toggle_open}
                    class={classes!(
                        "flex", "items-center", "gap-2",
                        "px-3", "py-1.5",
                        "rounded-md",
                        "text-sm", "font-medium",
                        "border",
                        if *is_open {
                            "border-amber-500/60 bg-amber-500/10 text-amber-300"
                        } else {
                            "border-zinc-700 bg-zinc-800 hover:bg-zinc-700 text-zinc-200"
                        },
                        "transition-colors", "duration-150"
                    )}
                    type={"button"}
                >
                    <span class={classes!("text-base", "leading-none")}>
                        { if *is_open { "−" } else { "+" } }
                    </span>
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

                if !items.is_empty() {
                    <ItemTray items={items.clone()} remove={remove.clone()} />
                }
            </div>

            // — Modal —
            {
                is_open.then_some(html! {
                    <>
                        <div
                            onclick={close_selector.clone()}
                            class={classes!(
                                "fixed", "inset-0", "z-40",
                                "bg-black/60", "backdrop-blur-sm"
                            )}
                        />

                        <div class={classes!(
                            "fixed", "inset-0", "z-50",
                            "flex", "items-center", "justify-center",
                            "p-4"
                        )}>
                            <div
                                ref={dropdown_ref.clone()}
                                onclick={stop_click}
                                class={classes!(
                                    "w-full", "max-w-5xl",
                                    "h-[560px]",
                                    "rounded-xl",
                                    "border", "border-zinc-700/80",
                                    "bg-zinc-950",
                                    "shadow-2xl", "shadow-black/60",
                                    "overflow-hidden",
                                    "grid", "grid-cols-[260px_1fr]"
                                )}
                            >
                                // ── Sidebar ──────────────────────────────
                                <aside class={classes!(
                                    "flex", "flex-col",
                                    "border-r", "border-zinc-800/80",
                                    "bg-zinc-900/60"
                                )}>
                                    // Header
                                    <div class={classes!(
                                        "flex", "items-center", "justify-between",
                                        "px-4", "py-3",
                                        "border-b", "border-zinc-800"
                                    )}>
                                        <span class={classes!(
                                            "text-xs", "font-semibold",
                                            "tracking-widest", "uppercase",
                                            "text-zinc-400"
                                        )}>{"Stat Filters"}</span>
                                        if has_filters {
                                            <button
                                                onclick={clear_filters}
                                                class={classes!(
                                                    "px-2", "py-0.5",
                                                    "text-xs", "rounded",
                                                    "border", "border-zinc-700",
                                                    "bg-zinc-800", "hover:bg-zinc-700",
                                                    "text-zinc-400", "hover:text-zinc-200",
                                                    "transition-colors"
                                                )}
                                                type={"button"}
                                            >
                                                {format!("Clear {n}", n = selected_stats.len())}
                                            </button>
                                        }
                                    </div>

                                    // OR / AND toggle
                                    <div class={classes!("px-3", "pt-2.5", "pb-2")}>
                                        <button
                                            onclick={on_toggle_mode}
                                            class={classes!(
                                                "w-full",
                                                "flex", "items-center", "justify-between",
                                                "px-3", "py-2",
                                                "rounded-md",
                                                "border",
                                                "border-zinc-700/60",
                                                "bg-zinc-800/60",
                                                "hover:bg-zinc-800",
                                                "transition-colors", "duration-150"
                                            )}
                                            type={"button"}
                                        >
                                            <span class={classes!("text-xs", "text-zinc-400")}>{"Match mode"}</span>
                                            <span class={classes!(
                                                "px-2", "py-0.5",
                                                "rounded",
                                                "text-xs", "font-bold", "tracking-wider",
                                                if *strict_or_mode {
                                                    "bg-amber-500/20 text-amber-300 border border-amber-500/40"
                                                } else {
                                                    "bg-zinc-700 text-zinc-300 border border-zinc-600"
                                                }
                                            )}>
                                                { if *strict_or_mode { "OR" } else { "AND" } }
                                            </span>
                                        </button>
                                    </div>

                                    // Stat chip grid
                                    <div class={classes!("flex-1", "overflow-auto", "px-3", "pb-3")}>
                                        <div class={classes!("grid", "grid-cols-2", "gap-1")}>
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
                                                            "flex", "items-center", "gap-1.5",
                                                            "px-2", "py-1.5",
                                                            "rounded-md",
                                                            "border",
                                                            "text-left",
                                                            "transition-colors", "duration-100",
                                                            if is_selected {
                                                                "border-amber-500/50 bg-amber-500/10 text-amber-200"
                                                            } else {
                                                                "border-zinc-800 bg-zinc-900 hover:bg-zinc-800 text-zinc-400 hover:text-zinc-200"
                                                            }
                                                        )}
                                                        type={"button"}
                                                        title={stat.to_string()}
                                                    >
                                                        <Image
                                                            src={ImageType::StatsFilter(stat)}
                                                            class={classes!("w-4", "h-4", "shrink-0")}
                                                        />
                                                        <span class={classes!(
                                                            "text-[11px]", "leading-tight", "truncate"
                                                        )}>
                                                            {stat.to_string()}
                                                        </span>
                                                    </button>
                                                }
                                            })}
                                        </div>
                                    </div>
                                </aside>

                                // ── Main panel ───────────────────────────
                                <section class={classes!("flex", "flex-col", "min-w-0", "bg-zinc-950")}>

                                    // Search bar
                                    <div class={classes!(
                                        "flex", "items-center", "gap-2",
                                        "px-3", "py-2.5",
                                        "border-b", "border-zinc-800"
                                    )}>
                                        <span class={classes!("text-zinc-500", "text-sm", "shrink-0")}>{"⌕"}</span>
                                        <input
                                            value={(*search).clone()}
                                            oninput={on_search}
                                            placeholder={"Search items..."}
                                            class={classes!(
                                                "flex-1", "min-w-0",
                                                "bg-transparent",
                                                "text-sm", "text-zinc-200",
                                                "placeholder:text-zinc-600",
                                                "outline-none"
                                            )}
                                        />
                                        <button
                                            onclick={close_selector}
                                            class={classes!(
                                                "flex", "items-center", "justify-center",
                                                "w-7", "h-7",
                                                "rounded-md", "shrink-0",
                                                "border", "border-zinc-700",
                                                "bg-zinc-800", "hover:bg-zinc-700",
                                                "text-zinc-400", "hover:text-zinc-200",
                                                "text-sm", "transition-colors"
                                            )}
                                            type={"button"}
                                        >{"✕"}</button>
                                    </div>

                                    // Selected tray (inline, compact)
                                    if !items.is_empty() {
                                        <div class={classes!(
                                            "flex", "items-center", "gap-2",
                                            "px-3", "py-2",
                                            "border-b", "border-zinc-800/80",
                                            "bg-zinc-900/40"
                                        )}>
                                            <span class={classes!(
                                                "text-[11px]", "font-mono",
                                                "text-zinc-500", "shrink-0", "w-8"
                                            )}>
                                                {format!("{n}×", n = items.len())}
                                            </span>
                                            <div class={classes!("flex-1", "overflow-x-auto")}>
                                                <ItemTray
                                                    items={items.clone()}
                                                    remove={remove.clone()}
                                                />
                                            </div>
                                        </div>
                                    }

                                    // Results count
                                    <div class={classes!(
                                        "flex", "items-center", "gap-2",
                                        "px-3", "py-1.5",
                                        "border-b", "border-zinc-800/60"
                                    )}>
                                        <span class={classes!(
                                            "text-[11px]", "font-mono", "text-zinc-500"
                                        )}>
                                            { format!("{n} item(s)", n = filtered_items.len()) }
                                        </span>
                                        if has_filters {
                                            <span class={classes!(
                                                "text-[10px]", "px-1.5", "py-0.5",
                                                "rounded",
                                                "bg-amber-500/10", "border", "border-amber-500/20",
                                                "text-amber-400/80"
                                            )}>
                                                { format!("{n} filter(s) active", n = selected_stats.len()) }
                                            </span>
                                        }
                                    </div>

                                    // Item list
                                    <div class={classes!("flex-1", "overflow-auto")}>
                                        <div class={classes!("grid", "grid-cols-1", "p-2", "gap-0.5")}>
                                            {for filtered_items.iter().copied().map(|item| {
                                                let onclick = {
                                                    let insert = insert.clone();
                                                    Callback::from(move |_| insert.emit(item))
                                                };

                                                html! {
                                                    <button
                                                        key={item.name()}
                                                        {onclick}
                                                        class={classes!(
                                                            "w-full",
                                                            "flex", "items-center", "gap-3",
                                                            "px-3", "py-2",
                                                            "rounded-md",
                                                            "text-left",
                                                            "hover:bg-zinc-800/80",
                                                            "group", "transition-colors", "duration-100"
                                                        )}
                                                        title={format!("Add {name}", name = item.name())}
                                                        type={"button"}
                                                    >
                                                        <div class={classes!(
                                                            "shrink-0", "rounded",
                                                            "border", "border-zinc-700/40",
                                                            "group-hover:border-amber-500/30",
                                                            "transition-colors"
                                                        )}>
                                                            <Image src={item.image_type()} class={classes!("w-8", "h-8", "block")} />
                                                        </div>
                                                        <span class={classes!(
                                                            "text-sm", "text-zinc-300",
                                                            "group-hover:text-zinc-100",
                                                            "truncate", "transition-colors"
                                                        )}>
                                                            {item.name()}
                                                        </span>
                                                        <span class={classes!(
                                                            "ml-auto", "shrink-0",
                                                            "text-zinc-700", "group-hover:text-amber-500/60",
                                                            "text-sm", "transition-colors"
                                                        )}>{"+"}</span>
                                                    </button>
                                                }
                                            })}

                                            {filtered_items.is_empty().then_some(html! {
                                                <div class={classes!(
                                                    "py-16", "text-center",
                                                    "text-sm", "text-zinc-600"
                                                )}>
                                                    <div class={classes!("text-3xl", "mb-2")}>{"∅"}</div>
                                                    {"No items match the current filters"}
                                                </div>
                                            })}
                                        </div>
                                    </div>
                                </section>
                            </div>
                        </div>
                    </>
                })
            }
        </div>
    }
}
