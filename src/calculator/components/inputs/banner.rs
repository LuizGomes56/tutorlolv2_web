use crate::components::{
    image::{Image, ImageType},
    selector::Selector,
};
use tutorlolv2::ChampionId;
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
                "z-10", "w-full", "h-full", "content-end"
            )}>
                <Selector<ChampionId>
                    value={champion_id}
                    callback={callback.clone()}
                    label_class={classes!("gap-2", "p-2", "overflow-hidden")}
                    img_class={classes!("w-8", "h-8", "shrink-0")}
                    input_class={classes!(
                        "font-bold", "text-lg", "text-std-400",
                        "text-shadow", "placeholder:text-white", "min-w-0",
                    )}
                    dropdown_class={classes!("px-1.5", "py-1", "w-full")}
                />
            </div>
        </div>
    }
}
