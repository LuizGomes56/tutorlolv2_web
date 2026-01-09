use crate::{components::image::Image, model::AbilityLevels, utils::ImageType};
use std::hint::unreachable_unchecked;
use tutorlolv2_gen::{AbilityId, AbilityName, ChampionId};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct AbilitiesProps {
    pub ability_levels: AbilityLevels,
    pub callback: Callback<AbilityLevels>,
    pub champion_id: ChampionId,
}

impl AbilityLevels {
    pub const fn field(&mut self, ability_id: AbilityId) -> &mut u8 {
        match ability_id {
            AbilityId::Q(_) => &mut self.q,
            AbilityId::W(_) => &mut self.w,
            AbilityId::E(_) => &mut self.e,
            AbilityId::R(_) => &mut self.r,
            _ => unsafe { unreachable_unchecked() },
        }
    }

    pub const fn get(&mut self, ability_id: AbilityId) -> u8 {
        *self.field(ability_id)
    }

    pub const fn set(mut self, ability_id: AbilityId, value: u8) -> Self {
        *self.field(ability_id) = value;
        self
    }
}

#[component]
pub fn Abilities(props: &AbilitiesProps) -> Html {
    let AbilitiesProps {
        mut ability_levels,
        champion_id,
        ..
    } = *props;

    let callback = &props.callback;

    [AbilityId::Q, AbilityId::W, AbilityId::E, AbilityId::R]
        .into_iter()
        .map(|func| {
            let ability_id = func(AbilityName::Void);
            let value = ability_levels.get(ability_id);
            html! {
                <label class={classes!("grid", "grid-cols-2")}>
                    <Image
                        src={ImageType::Ability(champion_id, ability_id.into())}
                        class={classes!("h-9", "w-9", "rounded")}
                    />
                    <input
                        value={value.to_string()}
                        placeholder={value.to_string()}
                        oninput={{
                            let callback = callback.clone();
                            Callback::from(move |e: InputEvent| {
                                let target = e.target_unchecked_into::<HtmlInputElement>();
                                let value = target.value().parse::<u8>().unwrap_or(0);
                                callback.emit(ability_levels.set(ability_id, value));
                            })
                        }}
                        type={"number"}
                        class={classes!(
                            "text-sm", "bg-std-800", "w-9",
                            "h-9", "text-center", "text-std-200"
                        )}
                    />
                </label>
            }
        })
        .collect::<Html>()
}
