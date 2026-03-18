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

    fn make_(
        va: impl IntoPropValue<Html>,
        ve: impl IntoPropValue<Html>,
        image: DragonImage,
    ) -> Html {
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

    html! {
        <div class={classes!("box", "overflow-auto")}>
            <table class={classes!("data-table")}>
                <thead>
                    <tr>
                        <th />
                        <th>{"Allies"}</th>
                        <th>{"Enemies"}</th>
                    </tr>
                </thead>
                <tbody>
                    {make_(dragons.ally_earth, dragons.enemy_earth, DragonImage::Earth)}
                    {make_(dragons.ally_fire, "\u{2013}", DragonImage::Fire)}
                    {make_("\u{2013}", "\u{2013}", DragonImage::Ocean)}
                    {make_("\u{2013}", "\u{2013}", DragonImage::Chemtech)}
                </tbody>
            </table>
        </div>
    }
}
