use crate::{
    calculator::FinalEnemy,
    components::image::Image,
    model::{Attacks, Damages, RangeDamage},
    overlay::Enemy,
    utils::{EnumCast, encode_offset},
};
use std::{collections::HashSet, ops::Range, rc::Rc};
use tutorlolv2_gen::{
    AbilityId, ChampionId, Ctx, DamageType, ITEM_IDENTS, ItemId, MergeData, RUNE_CLOSURES, RuneId,
    TypeMetadata,
};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TableBodyProps<T: PartialEq + 'static + DisplayDamage> {
    pub enemies: Rc<[T]>,
    pub ability_offsets: &'static [Range<usize>],
    pub abilities_meta: Rc<[TypeMetadata<AbilityId>]>,
    pub abilities_to_merge: Rc<[MergeData]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

pub trait DisplayDamage {
    fn get_eval_ctx(&self) -> &Ctx;
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
    fn get_eval_ctx(&self) -> &Ctx {
        &self.eval_ctx
    }
}

impl DisplayDamage for Enemy {
    fn get_damages(&self) -> &Damages {
        &self.damages
    }
    fn get_champion_id(&self) -> ChampionId {
        self.champion_id
    }
    fn get_eval_ctx(&self) -> &Ctx {
        &self.eval_ctx
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
        ability_offsets,
    } = props;

    enemies
        .iter()
        .map(|enemy| {
            let damages = enemy.get_damages();
            let champion_id = enemy.get_champion_id();
            let eval_ctx = enemy.get_eval_ctx();
            let eval_meta = unsafe {
                const VARIANTS: usize = size_of::<Ctx>() / size_of::<f32>();
                core::mem::transmute::<_, &[f32; VARIANTS]>(eval_ctx)
            };

            let abilities = {
                let damages = &damages.abilities;
                let mut data = damages.iter().map(|&v| (v, None)).collect::<Vec<_>>();

                let mut to_remove = HashSet::with_capacity(abilities_to_merge.len());

                for merge in abilities_to_merge.iter() {
                    let min_idx = merge.minimum_damage as usize;
                    let max_idx = merge.maximum_damage as usize;

                    if let (Some(min_val), Some(max_val)) =
                        (damages.get(min_idx), damages.get(max_idx))
                    {
                        if *max_val != 0 && min_val != max_val {
                            data[min_idx].1 = Some(max_idx);
                        }
                        to_remove.insert(max_idx);
                    }
                }

                data.into_iter()
                    .enumerate()
                    .filter_map(|(i, (min_val, max_idx))| {
                        if to_remove.contains(&i) {
                            return None;
                        }

                        let text = match max_idx {
                            Some(max_i) => {
                                let max_val = damages[max_i];
                                format!("{min_val} - {max_val}")
                            }
                            None => min_val.to_string(),
                        };

                        Some((i, max_idx, text))
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

            let rune_damages = damages
                .runes
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let damage_type = runes_meta[i].damage_type;
                    let offsets = RUNE_CLOSURES[runes_meta[i].kind.index()].clone();
                    let encoded = encode_offset(Some(offsets))
                        .as_ref()
                        .map(ToString::to_string);
                    html! {
                        <td key={i} data-offset-main={encoded} class={get_classes(damage_type)}>
                            {*item}
                        </td>
                    }
                })
                .collect::<Html>();

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

            let item_damages = items_meta
                .into_iter()
                .enumerate()
                .map(|(i, metadata)| {
                    let TypeMetadata {
                        kind, damage_type, ..
                    } = *metadata;
                    let minimum_damage = damages.items[i];
                    let maximum_damage = damages.items[i + 1];

                    let data = ITEM_IDENTS[kind.index()]
                        .into_iter()
                        .map(|ident| {
                            let value = eval_meta[*ident as usize];
                            format!("{ident}: {value}, ")
                        })
                        .collect::<String>();

                    html! {
                        <td
                            key={i}
                            data-eval={data}
                            class={get_classes(damage_type)}
                        >
                            {minimum_damage}{(
                                maximum_damage != 0
                                    && minimum_damage
                                    != maximum_damage
                                ).then_some(
                                html!(<>{" - "}{maximum_damage}</>)
                            )}
                        </td>
                    }
                })
                .collect::<Html>();

            html! {
                <tr>
                    <td>
                        <Image src={champion_id.image_type()} />
                    </td>
                    {attacks(basic_attack, DamageType::Physical)}
                    {attacks(critical_strike, DamageType::Physical)}
                    <td class={get_classes(DamageType::Mixed)}>
                        {onhit_min}{(onhit_max != 0 && onhit_max != onhit_min).then_some(
                            html!(<>{" - "}{onhit_max}</>)
                        )}
                    </td>
                    {abilities.into_iter().map(|(i, j, damage)| {
                        let damage_type = abilities_meta[i].damage_type;
                        let main_offset = encode_offset(Some(ability_offsets[i].clone()))
                            .as_ref()
                            .map(ToString::to_string);
                        let exc_offset = j.and_then(|j| {
                            encode_offset(Some(ability_offsets[j].clone()))
                                .as_ref()
                                .map(ToString::to_string)
                        });

                        html! {
                            <td
                                key={i}
                                data-offset-main={main_offset}
                                data-offset-exc={exc_offset}
                                class={get_classes(damage_type)}
                            >
                                {damage}
                            </td>
                        }
                    }).collect::<Html>()}
                    {item_damages}
                    {rune_damages}
                </tr>
            }
        })
        .collect::<Html>()
}
