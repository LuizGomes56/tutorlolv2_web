use std::rc::Rc;

use tutorlolv2_gen::ChampionId;
use yew::prelude::*;

use crate::components::image::{Image, ImageType};

#[derive(PartialEq, Properties)]
pub struct BannerProps {
    pub riot_id: Rc<str>,
    pub game_time: u32,
    pub champion_id: ChampionId,
}

#[component]
pub fn Banner(props: &BannerProps) -> Html {
    let BannerProps {
        ref riot_id,
        game_time,
        champion_id,
    } = *props;

    let minutes = game_time / 60;
    let seconds = game_time % 60;
    let time = format!("{minutes:02}m {seconds:02}s");

    html! {
        <div class={classes!("box")}>
            <Image
                class={classes!(
                    "img-clipped", "h-24", "sm:h-40",
                    "md:h-48", "lg:h-64", "xl:h-32"
                )}
                src={ImageType::Centered(champion_id)}
            />
            <span class={classes!(
                "flex", "font-bold", "items-center",
                "justify-between", "text-zinc-300",
                "p-4"
            )}>
                <p>
                    {riot_id}{" - "}{champion_id.name()}
                </p>
                <p>{time}</p>
            </span>
        </div>
    }
}
