use crate::{
    components::{
        image::{Image, ImageType},
        stack::StackValue,
        tables::body::Victim,
    },
    utils::encode_offset,
};
use std::rc::Rc;
use tutorlolv2_gen::{CastId, ignite};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct StackTableProps<T: Victim + PartialEq + 'static> {
    pub enemies: Rc<[T]>,
    pub stack: Box<[StackValue]>,
}

#[component]
pub fn StackTable<T: Victim + PartialEq + 'static>(props: &StackTableProps<T>) -> Html {
    let StackTableProps { enemies, stack } = props;

    html! {
        <table>
            <thead>
                <tr>
                    <th></th>
                    <th>{ "Damage" }</th>
                    <th>{ "Health" }</th>
                    <th>{ "% HP" }</th>
                </tr>
            </thead>
            <tbody>
                {
                    enemies.iter().map(|enemy| {
                        let damages = enemy.damages();
                        let enemy_id = enemy.champion_id();
                        let max_health = enemy.max_health();

                        let total = stack
                            .iter()
                            .copied()
                            .map(|value| match value {
                                StackValue::Ability(i, ..) => damages.abilities[i],
                                StackValue::Item(i, ..) => damages.items[i],
                                StackValue::Rune(i, ..) => damages.runes[i],
                                StackValue::BasicAttack => damages.attacks.basic_attack,
                                StackValue::CriticalStrike => damages.attacks.critical_strike,
                                StackValue::OnhitMin => damages.attacks.onhit_damage.minimum_damage,
                                StackValue::OnhitMax => damages.attacks.onhit_damage.maximum_damage,
                                StackValue::Ignite(i) => ignite(i),
                            })
                            .sum::<i32>();

                        let final_hp = max_health - total;
                        let hp_damage = ((total as f32 / max_health as f32) * 100.0) as i32;

                        html! {
                            <tr>
                                <td
                                    class={classes!("cursor-pointer")}
                                    data_offset={encode_offset(&[enemy_id.formula()])}
                                >
                                    <Image src={ImageType::from(enemy_id)} />
                                </td>
                                <td>{ total }</td>
                                <td>{ final_hp }</td>
                                <td>{ hp_damage }{ "%" }</td>
                            </tr>
                        }
                    })
                    .collect::<Html>()
                }
            </tbody>
        </table>
    }
}
