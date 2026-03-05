use crate::{
    components::{
        image::{Image, ImageType},
        selector::Selector,
    },
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

    html! {
        <div
            title={"Click on champion name to open champion selector"}
            class={classes!("bg-std-900", "box", "relative")}
        >
            <Image
                src={ImageType::Centered(champion_id)}
                class={classes!("clip", "h-36")}
            />
            <div class={classes!(
                "absolute", "left-0", "bottom-0",
                "z-10", "w-full"
            )}>
                <Selector<ChampionId>
                    value={champion_id}
                    callback={callback.clone()}
                    box_class={classes!("gap-2", "m-2")}
                    img_class={classes!("w-8", "h-8")}
                    input_class={classes!(
                        "font-bold", "text-lg", "text-std-400",
                        "text-shadow", "bg-transparent",
                        "focus:ring-0", "focus:outline-none",
                        "placeholder:text-white"
                    )}
                    dropdown_class={classes!("p-1.5", "gap-1.5", "w-full")}
                />
            </div>
        </div>
    }
}
