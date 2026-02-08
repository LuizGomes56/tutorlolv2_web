use crate::{
    components::image::{Image, ImageType},
    impl_index,
    model::AbilityLevels,
};
use std::ops::{Index, IndexMut};
use tutorlolv2_gen::{AbilityId, AbilityName, ChampionId};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct AbilitiesProps {
    pub ability_levels: AbilityLevels,
    pub callback: Callback<AbilityLevels>,
    pub champion_id: ChampionId,
}

impl_index! {
    AbilityLevels[AbilityId] u8 {
        AbilityId::Q(_) => q,
        AbilityId::W(_) => w,
        AbilityId::E(_) => e,
        AbilityId::R(_) => r,
    }
}

#[component]
pub fn Abilities(props: &AbilitiesProps) -> Html {
    let AbilitiesProps {
        ability_levels,
        champion_id,
        ..
    } = *props;

    let callback = &props.callback;

    [AbilityId::Q, AbilityId::W, AbilityId::E, AbilityId::R]
        .into_iter()
        .map(|func| {
            let ability_id = func(AbilityName::Void);
            let value = ability_levels[ability_id];
            html! {
                <label class={classes!("grid", "grid-cols-2")}>
                    <Image
                        src={ImageType::Ability(champion_id, ability_id.into())}
                        class={classes!("h-8", "w-8", "rounded")}
                    />
                    <input
                        value={value.to_string()}
                        placeholder={value.to_string()}
                        oninput={{
                            let callback = callback.clone();
                            Callback::from(move |e: InputEvent| {
                                let target = e.target_unchecked_into::<HtmlInputElement>();
                                let value = target.value().parse::<u8>().unwrap_or(0);
                                let mut result = ability_levels;
                                result[ability_id] = value;
                                callback.emit(result);
                            })
                        }}
                        type={"number"}
                        class={classes!(
                            "text-sm", "bg-std-800", "w-8",
                            "h-8", "text-center", "text-std-200"
                        )}
                    />
                </label>
            }
        })
        .collect::<Html>()
}
