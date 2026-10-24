use super::item_clicks::ItemClicks;
use super::selector_dialog::{Search, SelectorDialog};
use crate::hover::Docs;
use strum::EnumCount;
use {
    crate::{
        calculator::{
            page::{EnemyProps, PlayerProps, TargetEntity},
            reducer::{DataAction, EnemyAction, LastAction, PlayerAction},
        },
        components::image::{Image, ImageType},
        utils::tray::{Tray, TrayAction, TrayEntry},
    },
    strum::VariantArray,
    tutorlolv2::{
        ChampionId, ItemId, Position, RuneId, StatName, ValueId,
        bitset::{BitSetArray, sizeof_bitset},
    },
    yew::prelude::*,
};

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

/// One active inventory; target selection belongs to the portrait tabs above it.
pub fn make_inventory<T>(
    champion_id: ChampionId,
    values: &Tray<T>,
    remove: Callback<u64>,
    recommendations: Callback<Tray<T>>,
) -> Html
where
    T: PartialEq + ValueId,
    ImageType: From<T>,
    Tray<T>: FromIterator<T>,
{
    let has_recommendations = Position::VARIANTS
        .iter()
        .any(|&position| !T::recommendations(champion_id, position).is_empty());
    html! {
        <section class="shop-inventory">
            <div class="inventory-heading"><h3>{"Inventory"}</h3><span role="status">{values.len()}{" selected"}</span></div>
            <div class="build-slots">
                {for values.iter().map(|&TrayEntry { id, value }| {
                    let remove = remove.clone();
                    html! {
                        <button key={id} type="button" class="loadout-slot is-equipped"
                            onclick={Callback::from(move |_| remove.emit(id))}
                            aria-label={format!("Remove {}", value.name())}
                            title={format!("{} · Click to remove", value.name())}
                            data_docs={Docs::global(value).encode()}>
                            <Image src={ImageType::from(value)} />
                            <span class="slot-remove" aria-hidden="true">{"×"}</span>
                        </button>
                    }
                })}
                {for (0..6usize.saturating_sub(values.len())).map(|i| html! {
                    <span key={i} class="loadout-slot is-empty" aria-hidden="true" />
                })}
            </div>
            <p class="inventory-help">{"Click an equipped icon to remove it."}</p>
            if has_recommendations {
                <div class="build-recommendations"><span>{"Recommended for your role"}</span><Recommendations<T> {champion_id} callback={recommendations} /></div>
            }
        </section>
    }
}

#[derive(PartialEq, Properties)]
pub struct RecommendationsProps<T: ValueId + PartialEq> {
    pub champion_id: ChampionId,
    pub callback: Callback<Tray<T>>,
}

#[component]
pub fn Recommendations<T>(props: &RecommendationsProps<T>) -> Html
where
    T: ValueId + PartialEq,
    Tray<T>: FromIterator<T>,
{
    html! {
        <div class="recommendation-roles">
            {for Position::VARIANTS.iter().map(|&position| {
                let callback = props.callback.clone();
                let array = T::recommendations(props.champion_id, position);
                let available = !array.is_empty();
                let label = format!("Apply recommended {} build for {}", format!("{position:?}").to_lowercase(), props.champion_id.name());
                html! {
                    <button type="button" onclick={Callback::from(move |_| callback.emit(array.iter().collect()))}
                        title={if available {label.clone()} else {format!("No {position:?} recommendations available")}}
                        aria-label={label} disabled={!available}>
                        <Image src={ImageType::Position(position)} />
                    </button>
                }
            })}
        </div>
    }
}

type FilterBitSet = BitSetArray<{ sizeof_bitset(StatName::COUNT) }>;
const FILTER_STATS: [StatName; 22] = [
    StatName::AttackDamage,
    StatName::AbilityPower,
    StatName::AttackSpeed,
    StatName::AbilityHaste,
    StatName::CritChance,
    StatName::CritDamage,
    StatName::ArmorPenetration,
    StatName::Lethality,
    StatName::MagicPenetration,
    StatName::Health,
    StatName::Armor,
    StatName::MagicResist,
    StatName::Mana,
    StatName::BaseManaRegen,
    StatName::BaseHealthRegen,
    StatName::LifeSteal,
    StatName::Omnivamp,
    StatName::MoveSpeed,
    StatName::HealAndShieldPower,
    StatName::Tenacity,
    StatName::GoldPer10Seconds,
    StatName::AdaptiveForce,
];

#[derive(PartialEq, Properties)]
pub struct ItemFilterProps {
    pub filters: UseStateHandle<FilterBitSet>,
}

#[component]
pub fn ItemFilter(props: &ItemFilterProps) -> Html {
    let filters = &props.filters;
    let toggle = use_callback(filters.clone(), move |stat: StatName, filters| {
        let mut next = **filters;
        next.toggle_const(stat as _);
        filters.set(next);
    });
    let clear = use_callback(filters.clone(), move |_: MouseEvent, filters| {
        filters.set(FilterBitSet::EMPTY)
    });
    html! {
        <nav class="selector-filters item-stat-rail" aria-label="Filter items by stats">
            <span class="filter-heading">{"Stats"}</span>
            <button type="button" class={classes!("stat-filter", "all-stat-filter", filters.is_empty_const().then_some("is-active"))}
                onclick={clear} aria-label="All items" title="All items" aria-pressed={filters.is_empty_const().to_string()}>
                <span class="all-filter-icon" aria-hidden="true">{"⊞"}</span><span>{"All"}</span>
            </button>
            {for FILTER_STATS.into_iter().map(|stat| {
                let toggle = toggle.clone(); let active = filters.contains_const(stat as _);
                html! {
                    <button type="button" class={classes!("stat-filter", active.then_some("is-active"))}
                        onclick={Callback::from(move |_| toggle.emit(stat))} aria-label={stat.name()} title={stat.name()} aria-pressed={active.to_string()}>
                        <Image src={ImageType::StatsFilter(stat)} />
                    </button>
                }
            })}
        </nav>
    }
}

fn percent_stat(stat: StatName) -> bool {
    matches!(
        stat,
        StatName::AttackSpeed
            | StatName::CritChance
            | StatName::CritDamage
            | StatName::ArmorPenetration
            | StatName::MagicPenetrationPercent
            | StatName::MoveSpeedPercent
            | StatName::LifeSteal
            | StatName::Omnivamp
            | StatName::BaseManaRegen
            | StatName::BaseHealthRegen
            | StatName::HealAndShieldPower
            | StatName::Tenacity
    )
}

/// Give long compound names a balanced break without changing their text.
fn item_name(name: &str) -> Html {
    name.split(' ')
        .enumerate()
        .map(|(i, word)| {
            let split = (word.chars().count() > 12)
                .then(|| word.char_indices().nth(word.chars().count() / 2).unwrap().0);
            html! {<>
                {if i > 0 { " " } else { "" }}
                if let Some(at) = split {
                    {&word[..at]}<wbr />{&word[at..]}
                } else { {word} }
            </>}
        })
        .collect()
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
    let selected = use_state(|| ItemId::AbyssalMask);
    let item_clicks = use_mut_ref(ItemClicks::default);
    let player_remove = use_item_tray_callback(player_props, TrayAction::RemoveById);
    let player_replace = use_item_tray_callback(player_props, TrayAction::Replace);
    let target = **entity;
    {
        let item_clicks = item_clicks.clone();
        use_effect_with(
            (**is_open, target, query.clone(), filters.clone()),
            move |_| {
                item_clicks.borrow_mut().reset();
            },
        );
    }
    let (items, champion_id) = match target {
        TargetEntity::Player => (
            &player_props.player.data.items,
            player_props.player.data.champion_id,
        ),
        TargetEntity::Enemy(i) => (
            &enemy_props.enemies[i].items,
            enemy_props.enemies[i].champion_id,
        ),
    };
    let remove = match target {
        TargetEntity::Player => player_remove,
        TargetEntity::Enemy(i) => {
            let props = enemy_props.clone();
            Callback::from(move |id| {
                props.last_action.replace(LastAction::EnemyPlayer(i));
                props.enemies.dispatch(EnemyAction::Change(
                    i,
                    DataAction::Tray(TrayAction::RemoveById(id)),
                ));
            })
        }
    };
    let replace = match target {
        TargetEntity::Player => player_replace,
        TargetEntity::Enemy(i) => {
            let props = enemy_props.clone();
            Callback::from(move |tray| {
                props.last_action.replace(LastAction::EnemyPlayer(i));
                props.enemies.dispatch(EnemyAction::Change(
                    i,
                    DataAction::Tray(TrayAction::Replace(tray)),
                ));
            })
        }
    };
    let insert = {
        let player_props = player_props.clone();
        let enemy_props = enemy_props.clone();
        Callback::from(move |item| match target {
            TargetEntity::Player => {
                player_props.last_action.replace(LastAction::CurrentPlayer);
                player_props
                    .player
                    .dispatch(PlayerAction::Data(DataAction::Tray(TrayAction::Insert(
                        item,
                    ))));
            }
            TargetEntity::Enemy(i) => {
                enemy_props.last_action.replace(LastAction::EnemyPlayer(i));
                enemy_props.enemies.dispatch(EnemyAction::Change(
                    i,
                    DataAction::Tray(TrayAction::Insert(item)),
                ));
            }
        })
    };
    let normalized_query = query.trim().to_ascii_lowercase();
    let options = ItemId::VARIANTS
        .iter()
        .copied()
        .filter(|item| {
            !item.data().custom
                && item.data().purchasable
                && item.has_map(tutorlolv2::GameMap::SummonersRift)
                && item.name().to_ascii_lowercase().contains(&normalized_query)
                && (filters.is_empty_const()
                    || filters.iter_const().all(|v| {
                        ItemId::filter(StatName::from_repr(v as _).unwrap()).contains(item)
                    }))
        })
        .collect::<Vec<_>>();
    let reset = {
        let query = query.clone();
        let filters = filters.clone();
        Callback::from(move |_| {
            query.set(String::new());
            filters.set(FilterBitSet::EMPTY);
        })
    };
    let add_selected = {
        let insert = insert.clone();
        let item_clicks = item_clicks.clone();
        let item = *selected;
        Callback::from(move |_| {
            item_clicks.borrow_mut().reset();
            insert.emit(item);
        })
    };
    let total_cost: u32 = items.iter().map(|e| u32::from(e.value.price())).sum();
    let player_champion = player_props.player.data.champion_id;
    let oninteraction = {
        let item_clicks = item_clicks.clone();
        Callback::from(move |event: MouseEvent| {
            let catalog_click = event
                .target_dyn_into::<web_sys::Element>()
                .and_then(|element| element.closest(".item-card").ok().flatten())
                .is_some();
            if !catalog_click {
                item_clicks.borrow_mut().reset();
            }
        })
    };
    html! {
        <SelectorDialog title="Item shop" kind="items" is_open={is_open.clone()} {oninteraction}>
            <div class="selector-workspace">
                <ItemFilter filters={filters.clone()} />
                <main class="selector-catalog">
                    <div class="catalog-toolbar"><h3>{"All items"}<span class="catalog-count" role="status">{options.len()}</span></h3><Search query={query.clone()} placeholder="Search items…" /></div>
                    <div class="catalog-heading">
                        <div class="active-filter-list">
                            if filters.is_empty_const() { <span class="catalog-hint">{"Select an item to view its stats"}</span> }
                            {for FILTER_STATS.into_iter().filter(|&s| filters.contains_const(s as _)).map(|stat| html! {<span class="active-stat-name">{stat.name()}</span>})}
                        </div>
                        if !filters.is_empty_const() || !query.is_empty() { <button type="button" class="text-action" onclick={reset.clone()}>{"Reset filters"}</button> }
                    </div>
                    <div class="catalog-scroll">
                        if options.is_empty() {
                            <div class="catalog-empty"><h3>{"No items found"}</h3><p>{"Try another name or fewer stat filters."}</p><button type="button" class="empty-reset" onclick={reset}>{"Clear search & filters"}</button></div>
                        } else {
                            <div class="item-catalog-grid">
                                {for options.into_iter().map(|item| {
                                    let selected = selected.clone();
                                    let active = *selected == item;
                                    let count = items.iter().filter(|e| e.value == item).count();
                                    let onclick = {
                                        let insert = insert.clone(); let item_clicks = item_clicks.clone();
                                        Callback::from(move |event: MouseEvent| {
                                            selected.set(item);
                                            if event.detail() == 0 {
                                                item_clicks.borrow_mut().reset();
                                            } else if item_clicks.borrow_mut().click(item, target, event.time_stamp()) {
                                                insert.emit(item);
                                            }
                                        })
                                    };
                                    let ondblclick = {
                                        let insert = insert.clone(); let item_clicks = item_clicks.clone();
                                        Callback::from(move |event: MouseEvent| {
                                            if item_clicks.borrow_mut().native_double_click(item, target, event.time_stamp()) {
                                                insert.emit(item);
                                            }
                                        })
                                    };
                                    html! {
                                        <button type="button" key={item as u16} class={classes!("item-card", active.then_some("is-selected"), (count > 0).then_some("is-equipped"))}
                                            {onclick} {ondblclick}
                                            aria-label={item.name()} aria-pressed={active.to_string()} title={format!("{} · Double-click to add", item.name())}
                                            data_docs={Docs::global(item).encode()}>
                                            <span class="catalog-art"><Image src={ImageType::from(item)} />
                                                if count > 0 { <span class="equipped-badge" aria-label={format!("{count} equipped")}>{count}</span> }
                                            </span>
                                            <span class="item-card-name">{item_name(item.name())}</span>
                                            <span class="item-price"><Image class={classes!("gold-coin")} src={ImageType::StatsFilter(StatName::GoldPer10Seconds)} />{item.price()}</span>
                                        </button>
                                    }
                                })}
                            </div>
                        }
                    </div>
                    <p class="catalog-shortcut">{"Double-click an item to add it to your inventory."}</p>
                </main>
                <aside class="selector-builds" aria-label="Champion inventory and item details">
                    <nav class="champion-tabs" aria-label="Choose champion inventory">
                        <button type="button" class={classes!("champion-tab", "champion-player", (target == TargetEntity::Player).then_some("is-active"))}
                            onclick={{let entity = entity.clone(); Callback::from(move |_| entity.set(TargetEntity::Player))}}
                            aria-pressed={(target == TargetEntity::Player).to_string()} aria-label={format!("Edit You: {}", player_champion.name())}>
                            <Image src={ImageType::Champion(player_champion)} /><span><small>{"Your champion"}</small><strong>{player_champion.name()}</strong></span>
                            if target == TargetEntity::Player { <span class="target-active-mark" aria-hidden="true">{"✓"}</span> }
                        </button>
                        <div class="enemy-target-group"><span class="target-group-label">{"Enemies"}</span><div class="enemy-tabs">
                            {for enemy_props.enemies.iter().enumerate().map(|(i, enemy)| {
                                let entity = entity.clone(); let champion = enemy.champion_id; let enemy_target = TargetEntity::Enemy(i);
                                html! {
                                    <button type="button" class={classes!("champion-tab", "champion-enemy", (target == enemy_target).then_some("is-active"))}
                                        onclick={Callback::from(move |_| entity.set(enemy_target))} aria-pressed={(target == enemy_target).to_string()}
                                        aria-label={format!("Edit Enemy {}: {}", i + 1, champion.name())} title={champion.name()}>
                                        <Image src={ImageType::Champion(champion)} /><span>{i + 1}</span>
                                    </button>
                                }
                            })}
                        </div></div>
                    </nav>
                    <div class="shop-detail-scroll">
                        <section class="selected-item" aria-label="Selected item">
                            <div class="selected-item-heading" data_docs={Docs::global(*selected).encode()}>
                                <Image src={ImageType::from(*selected)} /><div><h3>{selected.name()}</h3><span class="item-price"><Image class={classes!("gold-coin")} src={ImageType::StatsFilter(StatName::GoldPer10Seconds)} />{selected.price()}</span></div>
                            </div>
                            <dl class="selected-item-stats">
                                {for selected.stats().iter().map(|&(stat, value)| html! {
                                    <div><dt><Image src={ImageType::StatsFilter(stat)} />{stat.name()}</dt><dd>{format!("+{value}{}", if percent_stat(stat) { "%" } else { "" })}</dd></div>
                                })}
                            </dl>
                            <button type="button" class="shop-add-item" onclick={add_selected}>{"Add item"}<span aria-hidden="true">{"+"}</span></button>
                        </section>
                        <div class="shop-active-champion"><Image src={ImageType::Champion(champion_id)} /><span><small>{if target == TargetEntity::Player { "Your build" } else { "Enemy build" }}</small><strong>{champion_id.name()}</strong></span><span class="build-cost"><Image class={classes!("gold-coin")} src={ImageType::StatsFilter(StatName::GoldPer10Seconds)} />{total_cost}</span></div>
                        {make_inventory(champion_id, items, remove, replace)}
                    </div>
                </aside>
            </div>
        </SelectorDialog>
    }
}
