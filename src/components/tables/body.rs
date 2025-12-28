use crate::{
    calculator::FinalEnemy, components::image::Image, model::RangeDamage, utils::ImageType,
};
use std::rc::Rc;
use tutorlolv2_gen::{AbilityId, DamageType, ItemId, MergeData, RuneId, TypeMetadata};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TableBodyProps {
    pub enemies: Rc<[FinalEnemy]>,
    pub abilities_meta: Rc<[TypeMetadata<AbilityId>]>,
    pub abilities_to_merge: Rc<[MergeData]>,
    pub items_meta: Rc<[TypeMetadata<ItemId>]>,
    pub runes_meta: Rc<[TypeMetadata<RuneId>]>,
}

#[component]
pub fn TableBody(props: &TableBodyProps) -> Html {
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
            let FinalEnemy {
                damages,
                champion_id,
                ..
            } = enemy;

            let abilities = {
                let mut result = damages
                    .abilities
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();

                let mut indexes = Vec::new();

                for merge in abilities_to_merge.iter() {
                    indexes.push(merge.maximum_damage as usize);
                    let max = &result[merge.maximum_damage as usize];
                    let min = &result[merge.minimum_damage as usize];
                    if max != "0" && max != min {
                        let damage = format!(" - {max}");
                        result[merge.minimum_damage as usize].push_str(&damage);
                    }
                }

                indexes.sort_unstable();
                indexes.dedup();
                indexes.reverse();

                for i in indexes {
                    if i < result.len() {
                        result.remove(i);
                    }
                }

                result
            };

            fn get_classes(damage_type: DamageType) -> Classes {
                classes!("text-sm", "text-center", &format!("{damage_type:?}"))
            }

            fn cell<T, D, V>(metadata: &Rc<[TypeMetadata<T>]>, damages: D) -> Html
            where
                V: ToString,
                D: IntoIterator<Item = V>,
            {
                damages
                    .into_iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let damage_type = metadata[i].damage_type;
                        html! {
                            <td class={get_classes(damage_type)}>
                                {item.to_string()}
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

            let RangeDamage {
                minimum_damage: onhit_min,
                maximum_damage: onhit_max,
            } = damages.attacks.onhit_damage;

            html! {
                <tr>
                    <Image src={ImageType::from(champion_id)} />
                    {attacks(damages.attacks.basic_attack, DamageType::Physical)}
                    {attacks(damages.attacks.critical_strike, DamageType::Physical)}
                    <td class={get_classes(DamageType::Mixed)}>
                        {onhit_min}{(onhit_max != 0 && onhit_max != onhit_min).then_some({
                            html!(<>{" - "}{onhit_max}</>)
                        })}
                    </td>
                    {cell(abilities_meta, abilities)}
                    {cell(items_meta, &damages.items)}
                    {cell(runes_meta, &damages.runes)}
                </tr>
            }
        })
        .collect::<Html>()
}
