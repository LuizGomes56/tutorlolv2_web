use crate::{
    components::{
        image::{Image, ImageType},
        stack::{Stack, StackValue},
        tables::body::Victim,
    },
    utils::encode_offset,
};
use std::rc::Rc;
use tutorlolv2_gen::{CastId, ignite};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct StackTableProps<T: Victim + PartialEq + 'static> {
    #[prop_or_default]
    pub callback: Option<Callback<usize>>,
    pub enemies: Rc<[T]>,
    pub stack: Stack,
    #[prop_or_default]
    pub index: Option<usize>,
    pub level: u8,
}

#[component]
pub fn StackTable<T: Victim + PartialEq + 'static>(props: &StackTableProps<T>) -> Html {
    let StackTableProps {
        ref callback,
        ref enemies,
        ref stack,
        index,
        level,
    } = *props;

    html! {
        <table class={classes!("data-table", if index.is_some() { Some("overlay") } else { None })}>
            <thead>
                <tr>
                    <th class={classes!("w-0")}></th>
                    <th>{ "Damage" }</th>
                    <th>{ "Health" }</th>
                    <th>{ "% HP" }</th>
                </tr>
            </thead>
            <tbody>
                {
                    enemies.iter().enumerate().filter_map(|(i, enemy)| {
                        if let Some(j) = index && i != j {
                            return None;
                        }

                        let damages = enemy.damages();
                        let enemy_id = enemy.champion_id();
                        let max_health = enemy.max_health();

                        let total = stack
                            .iter()
                            .map(|entry| match entry.value {
                                StackValue::Ability { slot, .. } => damages.abilities[slot],
                                StackValue::Item(j, ..) => damages.items[j],
                                StackValue::Rune(j, ..) => damages.runes[j],
                                StackValue::BasicAttack => damages.attacks.basic_attack,
                                StackValue::CritStrike => damages.attacks.critical_strike,
                                StackValue::OnhitMin => damages.attacks.onhit_damage.minimum_damage,
                                StackValue::OnhitMax => damages.attacks.onhit_damage.maximum_damage,
                                StackValue::Ignite => ignite(level),
                            })
                            .sum::<i32>();

                        let final_hp = max_health - total;
                        let hp_damage = ((total as f32 / max_health as f32) * 100.0) as i32;

                        Some(html! {
                            <tr>
                                <td
                                    class={classes!("w-12")}
                                    data_offset={encode_offset(&[enemy_id.formula()])}
                                >
                                    <button
                                        class={classes!(
                                            "cursor-pointer",
                                            "outline-none",
                                            "focus:ring-1",
                                            "focus:ring-blue-500/75"
                                        )}
                                        onclick={callback.clone().map(|f| Callback::from(move |_| f.emit(i)))}
                                    >
                                        <Image src={ImageType::from(enemy_id)} />
                                    </button>
                                </td>
                                <td>{ total }</td>
                                <td>{ final_hp }</td>
                                <td>{ hp_damage }{ "%" }</td>
                            </tr>
                        })
                    })
                    .collect::<Html>()
                }
            </tbody>
        </table>
    }
}
