use crate::components::image::{Image, ImageType};
use tutorlolv2::L_TWRD;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TurretTableProps {
    pub damages: Html,
}

#[component]
pub fn TurretTable(props: &TurretTableProps) -> Html {
    let header = use_memo((), |_| {
        let head = |i| {
            html! {
                <th>
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
        <table class={classes!("data-table")}>
            {(*header).clone()}
            <tbody><tr>{props.damages.clone()}</tr></tbody>
        </table>
    }
}
