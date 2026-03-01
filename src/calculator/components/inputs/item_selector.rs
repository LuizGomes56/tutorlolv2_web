use crate::{
    calculator::{
        components::inputs::{enemies::use_enemy_data_callback, player::use_data_callback},
        page::{EnemyProps, PlayerProps},
        reducer::DataAction,
    },
    components::image::{Image, ImageType},
    utils::encode_offset,
};
use tutorlolv2_gen::{CastId, ItemId, StatName};
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TargetEntity {
    Player,
    Enemy(usize),
}

#[derive(PartialEq, Properties)]
pub struct ItemSelectorProps {
    pub player_props: PlayerProps,
    pub enemy_props: EnemyProps,
}

#[component]
pub fn ItemSelector(props: &ItemSelectorProps) -> Html {
    let ItemSelectorProps {
        player_props,
        enemy_props,
    } = props;

    let player_insert_callback = use_data_callback(player_props, DataAction::InsertItem);
    let player_remove_callback = use_data_callback(player_props, DataAction::RemoveItem);
    let player_recommendations = use_data_callback(player_props, DataAction::SetItemVec);

    let enemy_insert_callback = use_enemy_data_callback(enemy_props, DataAction::InsertItem);
    let enemy_remove_callback = use_enemy_data_callback(enemy_props, DataAction::RemoveItem);
    let enemy_recommendations = use_enemy_data_callback(enemy_props, DataAction::SetItemVec);

    let player = &player_props.player;
    let enemies = &enemy_props.enemies;

    let entity = use_state(|| TargetEntity::Player);

    let (insert, remove, recommendations) = match &*entity {
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

    let set_player = {
        let entity = entity.clone();
        use_callback((), move |_: MouseEvent, _| {
            entity.set(TargetEntity::Player);
        })
    };

    let set_enemy = {
        let entity = entity.clone();
        use_callback((), move |index: usize, _| {
            entity.set(TargetEntity::Enemy(index));
        })
    };

    let items = match &*entity {
        TargetEntity::Player => &player.data.items,
        TargetEntity::Enemy(index) => &enemies[*index].items,
    };

    let tray = use_memo((remove.clone(), items.clone()), |(remove, items)| {
        items
            .iter()
            .enumerate()
            .map(|(i, &item_id)| {
                let data_offset = encode_offset(&[item_id.formula()]);
                html! {
                    <button
                        {data_offset}
                        class={classes!("flex", "items-center", "gap-2", "p-2")}
                        onclick={{
                            let remove = remove.clone();
                            Callback::from(move |_| {
                                remove.emit(i);
                            })
                        }}
                    >
                        <Image src={ImageType::from(item_id)} />
                    </button>
                }
            })
            .collect::<Html>()
    });

    let options = use_memo(insert.clone(), |insert| {
        ItemId::VALUES
            .iter()
            .map(|&item_id| {
                let data_offset = encode_offset(&[item_id.formula()]);
                html! {
                    <button
                        {data_offset}
                        class={classes!(
                            "flex", "items-center", "gap-2",
                            "p-2"
                        )}
                        onclick={{
                            let insert = insert.clone();
                            Callback::from(move |_| {
                                insert.emit(item_id);
                            })
                        }}
                    >
                        <Image src={ImageType::from(item_id)} />
                        <span>{item_id.name()}</span>
                    </button>
                }
            })
            .collect::<Html>()
    });

    let filters = use_memo((), |_| {
        [
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
        ]
        .into_iter()
        .map(|stat| {
            html! {
                <Image
                    class={classes!("w-4", "h-4")}
                    src={ImageType::StatsFilter(stat)}
                />
            }
        })
        .collect::<Html>()
    });

    html! {
        <div hidden={true}>
            {(*filters).clone()}
            <div class={classes!("flex", "items-center", "gap-2")}>
                <span>{ "Self: " }</span>
                <button
                    class={classes!(
                        "border",
                        match &*entity {
                            TargetEntity::Player => "border-emerald-400",
                            _ => "border-transparent",
                        }
                    )}
                    onclick={set_player}
                >
                    <Image src={ImageType::from(player.data.champion_id)} />
                </button>
            </div>
            <div class={classes!("flex", "items-center", "gap-2")}>
                { "Enemies: " }
                for (i, enemy) in enemies.iter().enumerate() {
                    <button
                        class={classes!(
                            "border",
                            match &*entity {
                                TargetEntity::Enemy(j) if j == &i => "border-orange-400",
                                _ => "border-transparent",
                            }
                        )}
                        onclick={{
                            let set_enemy = set_enemy.clone();
                            Callback::from(move |_| {
                                set_enemy.emit(i);
                            })
                        }}
                    >
                        <Image src={ImageType::from(enemy.champion_id)} />
                    </button>
                }
            </div>
            {(*tray).clone()}
            <div class={classes!(
                "flex", "flex-col", "gap-3"
            )}>
                {(*options).clone()}
            </div>
        </div>
    }
}
