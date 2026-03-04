use crate::{
    components::image::{Image, ImageType},
    utils::encode_offset,
};
use tutorlolv2_gen::{CastId, ChampionId};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct BannerProps {
    pub callback: Callback<ChampionId>,
    pub champion_id: ChampionId,
}

#[component]
pub fn Banner(props: &BannerProps) -> Html {
    let BannerProps {
        champion_id,
        ref callback,
    } = *props;

    let data_offset = encode_offset(&[champion_id.formula()]);

    html! {
        <div
            {data_offset}
            class={classes!("bg-std-900", "box", "relative")}
        >
            <Image
                src={ImageType::Centered(champion_id)}
                class={classes!("clip", "h-36")}
            />
            <span class={classes!(
                "absolute", "left-4", "bottom-4", "text-shadow",
                "font-bold", "text-lg", "text-white",
            )}>
                {format!("{champion_id:?}")}
            </span>
            // <div class={classes!("absolute", "z-10", "bg-std-900", "w-full")}>
            //     <Selector<ChampionId> callback={callback.clone()} />
            // </div>
        </div>
    }
}
