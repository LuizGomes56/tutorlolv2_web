use crate::components::image::{Image, ImageType};
use tutorlolv2_gen::Position;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct RecommendationsProps {
    pub callback: Callback<Position>,
}

#[component]
pub fn Recommendations(props: &RecommendationsProps) -> Html {
    let RecommendationsProps { callback } = props;

    let position = use_state(|| None::<Position>);

    let elements = Position::ARRAY
        .into_iter()
        .map(|pos| {
            let onclick = {
                let callback = callback.clone();
                let position = position.clone();
                Callback::from(move |_| {
                    position.set(Some(pos));
                    callback.emit(pos);
                })
            };
            let class = classes!(
                "place-items-center",
                "border",
                "aspect-square",
                match *position {
                    Some(p) if p == pos => "border-green-500",
                    _ => "border-transparent",
                }
            );
            html! (
                <button {class} {onclick}>
                    <Image
                        class={classes!("h-6", "w-6")}
                        src={ImageType::Position(pos)}
                    />
                </button>
            )
        })
        .collect::<Html>();

    html! {
        <div class={classes!(
            "grid", "grid-cols-5", "gap-2"
        )}>
            {elements}
        </div>
    }
}
