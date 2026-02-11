use crate::{
    components::image::{Image, ImageType},
    utils::encode_offset,
};
use tutorlolv2_gen::{L_TWRD, TOWER_DAMAGE_OFFSET};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TurretTableProps {
    pub damages: Html,
}

#[component]
pub fn TurretTable(props: &TurretTableProps) -> Html {
    let header = use_memo((), |_| {
        let offset = encode_offset(&[&TOWER_DAMAGE_OFFSET]);
        let head = |i| {
            let data_offset = offset.clone();
            html! {
                <th {data_offset}>
                    <div class={classes!(
                        "relative", "w-fit", "flex",
                        "items-center", "justify-center",
                        "place-self-center"
                    )}>
                        <Image src={ImageType::Tower} />
                        <span class={classes!(
                            "text-sm", "img-letter"
                        )}>
                            {i}
                        </span>
                    </div>
                </th>
            }
        };
        html! {
            <thead>
                <tr>
                    {(0..L_TWRD).into_iter().map(head).collect::<Html>()}
                </tr>
            </thead>
        }
    });

    html! {
        <table>
            {(*header).clone()}
            <tbody><tr>{props.damages.clone()}</tr></tbody>
        </table>
    }
}
