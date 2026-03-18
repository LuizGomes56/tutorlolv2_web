use crate::{
    components::image::{Image, ImageType},
    model::AbilityLevels,
};
use tutorlolv2_gen::ChampionId;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct AbilityLevelsDisplayProps {
    pub champion_id: ChampionId,
    pub ability_levels: AbilityLevels,
}

#[component]
pub fn AbilityLevelsDisplay(props: &AbilityLevelsDisplayProps) -> Html {
    let AbilityLevelsDisplayProps {
        champion_id,
        ability_levels,
    } = props;

    html! {
        <div class={classes!("box", "overflow-auto")}>
            <table class={classes!("data-table")}>
                <thead>
                    <tr>
                        <th></th>
                        <th>{"Level"}</th>
                    </tr>
                </thead>
                <tbody>
                    for key in AbilityLevels::ABILITIES {
                        <tr>
                            <td class={classes!("w-10")}>
                                <Image src={ImageType::Ability(*champion_id, key.into())} />
                            </td>
                            <td>{ability_levels[key]}</td>
                        </tr>
                    }
                </tbody>
            </table>
        </div>
    }
}
