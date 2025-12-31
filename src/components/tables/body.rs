use crate::{
    calculator::FinalEnemy,
    components::image::Image,
    model::{Attacks, Damages, RangeDamage},
    overlay::Enemy,
    utils::EnumCast,
};
use std::{collections::HashSet, rc::Rc};
use tutorlolv2_gen::{AbilityId, ChampionId, DamageType, ItemId, MergeData, RuneId, TypeMetadata};
use yew::{html::IntoPropValue, prelude::*};

#[derive(PartialEq, Properties)]
pub struct TableBodyProps<T: PartialEq + 'static + DisplayDamage> {
    pub enemies: Rc<[T]>,
    pub abilities_meta: Rc<[TypeMetadata<AbilityId>]>,
    pub abilities_to_merge: Rc<[MergeData]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

pub trait DisplayDamage {
    fn get_damages(&self) -> &Damages;
    fn get_champion_id(&self) -> ChampionId;
}

impl DisplayDamage for FinalEnemy {
    fn get_damages(&self) -> &Damages {
        &self.damages
    }
    fn get_champion_id(&self) -> ChampionId {
        self.champion_id
    }
}

impl DisplayDamage for Enemy {
    fn get_damages(&self) -> &Damages {
        &self.damages
    }
    fn get_champion_id(&self) -> ChampionId {
        self.champion_id
    }
}

#[component]
pub fn TableBody<T: PartialEq + 'static + DisplayDamage>(props: &TableBodyProps<T>) -> Html {
    let TableBodyProps {
        enemies,
        abilities_meta,
        abilities_to_merge,
        items_meta,
        runes_meta,
    } = props;

    enemies
        .iter()
        .map(|enemy| {
            let damages = enemy.get_damages();
            let champion_id = enemy.get_champion_id();

            let abilities = {
                let damages = &damages.abilities;
                let mut data: Vec<(i32, Option<i32>)> =
                    damages.iter().map(|&v| (v, None)).collect();

                let mut to_remove = HashSet::with_capacity(abilities_to_merge.len());

                for merge in abilities_to_merge.iter() {
                    let max_idx = merge.maximum_damage as usize;
                    let min_idx = merge.minimum_damage as usize;

                    if let (Some(max_val), Some(min_val)) =
                        (damages.get(max_idx), damages.get(min_idx))
                    {
                        if *max_val != 0 && max_val != min_val {
                            data[min_idx].1 = Some(*max_val);
                        }
                        to_remove.insert(max_idx);
                    }
                }

                data.into_iter()
                    .enumerate()
                    .filter_map(|(i, (min, max))| match to_remove.contains(&i) {
                        true => None,
                        false => {
                            let text = match max {
                                Some(max) => format!("{min} - {max}"),
                                _ => min.to_string(),
                            };
                            Some((i, text))
                        }
                    })
                    .collect::<Vec<_>>()
            };

            fn get_classes(damage_type: DamageType) -> Classes {
                classes!(
                    "text-xs",
                    "text-center",
                    match damage_type {
                        DamageType::Physical => "text-orange-500",
                        DamageType::Magic => "text-sky-500",
                        DamageType::Mixed => "text-indigo-500",
                        DamageType::True => "text-white",
                        DamageType::Adaptative => "text-purple-500",
                        DamageType::Unknown => "text-emerald-500",
                    }
                )
            }

            fn cell<'a, T, D, V>(metadata: &Rc<[TypeMetadata<T>]>, damages: D) -> Html
            where
                V: Copy + 'a + IntoPropValue<Html>,
                D: IntoIterator<Item = &'a V>,
            {
                damages
                    .into_iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let damage_type = metadata[i].damage_type;
                        html! {
                            <td key={i} class={get_classes(damage_type)}>
                                {*item}
                            </td>
                        }
                    })
                    .collect::<Html>()
            }

            let attacks = |value, damage_type| {
                html! {
                    <td class={get_classes(damage_type)}>
                        {value}
                    </td>
                }
            };

            let Attacks {
                basic_attack,
                critical_strike,
                onhit_damage:
                    RangeDamage {
                        minimum_damage: onhit_min,
                        maximum_damage: onhit_max,
                    },
            } = damages.attacks;

            html! {
                <tr>
                    <Image src={champion_id.image_type()} />
                    {attacks(basic_attack, DamageType::Physical)}
                    {attacks(critical_strike, DamageType::Physical)}
                    <td class={get_classes(DamageType::Mixed)}>
                        {onhit_min}{(onhit_max != 0 && onhit_max != onhit_min).then_some({
                            html!(<>{" - "}{onhit_max}</>)
                        })}
                    </td>
                    {abilities.into_iter().map(|(i, damage)| {
                        let damage_type = abilities_meta[i].damage_type;
                        html! { <td key={i} class={get_classes(damage_type)}>{damage}</td> }
                    }).collect::<Html>()}
                    {cell(items_meta, &damages.items)}
                    {cell(runes_meta, &damages.runes)}
                </tr>
            }
        })
        .collect::<Html>()
}
