use crate::{
    components::image::{Image, ImageType},
    impl_index,
    model::AbilityLevels,
    utils::ReduceApply,
};
use std::ops::{Index, IndexMut};
use tutorlolv2::{ChampionId, Key};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct AbilitiesProps {
    pub ability_levels: AbilityLevels,
    pub callback: Callback<<AbilityLevels as ReduceApply>::Action>,
    pub champion_id: ChampionId,
}

impl_index! {
    AbilityLevels[Key] u8 {
        Key::Q => q,
        Key::W => w,
        Key::E => e,
        Key::R => r,
    }
}

#[component]
pub fn Abilities(props: &AbilitiesProps) -> Html {
    let AbilitiesProps {
        ability_levels,
        champion_id,
        ref callback,
    } = *props;

    AbilityLevels::ABILITIES
        .into_iter()
        .enumerate()
        .map(|(i, key)| {
            let value = ability_levels[key];
            let prototype = AbilityLevels::ACTIONS[i];
            let id = AttrValue::from(key.as_char().to_string());

            html! {
                <label for={&id} class={classes!("grid", "grid-cols-2")}>
                    <Image
                        src={ImageType::Ability(champion_id, key.into())}
                        class={classes!("flex", "items-center", "justify-center")}
                    />
                    <input
                        id={id}
                        value={value.to_string()}
                        placeholder={value.to_string()}
                        oninput={{
                            let callback = callback.clone();
                            Callback::from(move |e: InputEvent| {
                                let target = e.target_unchecked_into::<HtmlInputElement>();
                                let value = target.value().parse::<u8>().unwrap_or(0);
                                callback.emit(prototype(value));
                            })
                        }}
                        type={"number"}
                        class={classes!(
                            "text-sm", "bg-std-800",
                            "text-center", "text-std-200"
                        )}
                    />
                </label>
            }
        })
        .collect::<Html>()
}
