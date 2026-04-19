use crate::{
    components::image::{DragonImage, Image, ImageType, OtherImage},
    model::Dragons,
};
use yew::{html::IntoPropValue, prelude::*};

#[derive(PartialEq, Properties)]
pub struct DragonDisplayProps {
    pub dragons: Dragons,
}

#[component]
pub fn DragonDisplay(props: &DragonDisplayProps) -> Html {
    let DragonDisplayProps { dragons } = props;

    fn row(va: impl IntoPropValue<Html>, ve: impl IntoPropValue<Html>, image: DragonImage) -> Html {
        html! {
            <tr>
                <td class={classes!("w-10")}>
                    <Image src={ImageType::Other(OtherImage::Dragon(image))} />
                </td>
                <td>{va}</td>
                <td>{ve}</td>
            </tr>
        }
    }

    const UNKNOWN: &str = "\u{2013}";

    html! {
        <div class={classes!("box", "overflow-auto")}>
            <table class={classes!("data-table")}>
                <thead>
                    <tr>
                        <th />
                        <th>{"Allies"}</th>
                        <th>
                            <span class={classes!("pr-2")}>
                                {"Enemies"}
                            </span>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    {row(dragons.ally_earth, dragons.enemy_earth, DragonImage::Earth)}
                    {row(dragons.ally_fire, UNKNOWN, DragonImage::Fire)}
                    {row(UNKNOWN, UNKNOWN, DragonImage::Ocean)}
                    {row(UNKNOWN, UNKNOWN, DragonImage::Chemtech)}
                </tbody>
            </table>
        </div>
    }
}
