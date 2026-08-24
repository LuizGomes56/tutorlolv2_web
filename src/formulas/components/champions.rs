use crate::{
    components::{
        h2::H2,
        image::{Image, ImageType},
        selector::Selector,
    },
    formulas::components::code::Code,
    utils::{EnumCast, use_setter},
};
use tutorlolv2::{CastId, ChampionId, Position};
use yew::prelude::*;

#[component]
pub fn ChampionFormulas() -> Html {
    let champion = use_state(ChampionId::random);
    let callback = use_setter(&champion);

    fn get_recommendations<T, F>(f: F) -> Html
    where
        F: Fn(Position) -> &'static [T],
        T: CastId,
        ImageType: From<T>,
    {
        html! {
            for position in Position::ARRAY {
                <td class={"content-baseline"}>
                    <div class={classes!("flex", "flex-col", "gap-2", "py-2")}>
                        for item in f(position) {
                            <div
                                class={classes!(
                                    "justify-items-center",
                                    "md:justify-items-start",
                                    "overflow-hidden"
                                )}
                            >
                                <div class={classes!("flex", "items-center", "gap-3")}>
                                    <Image
                                        class={classes!("w-7", "h-7")}
                                        src={ImageType::from(*item)}
                                    />
                                    <span class={classes!(
                                        "text-std-300", "truncate",
                                        "hidden", "md:inline"
                                    )}>
                                        {item.name()}
                                    </span>
                                </div>
                            </div>
                        }
                    </div>
                </td>
            }
        }
    }

    html! {
        <div class={classes!("flex", "flex-col", "gap-6", "p-6", "box")}>
            <H2 text={"Champions"} />
            <p class={classes!("text-std-400")}>
                {concat!(
                    "Documentation for formulas being used to calculate the damage of ",
                    "a champion's abilities, its recommended items, runes per position, ",
                    "and other basic information about it"
                )}
            </p>
            <Selector<ChampionId>
                value={*champion}
                {callback}
            />
            <H2 text={"Recommended items and runes per position"} />
            <div class={classes!("overflow-auto")}>
                <table class={classes!("table-fixed")}>
                    <thead>
                        <tr>
                            for position in Position::ARRAY {
                                <th class={classes!("overflow-hidden")}>
                                    <div class={classes!(
                                        "flex", "items-center", "gap-3",
                                        "justify-self-center",
                                        "md:justify-self-start"
                                    )}>
                                        <Image
                                            class={classes!("w-8", "h-8")}
                                            src={ImageType::Position(position)}
                                        />
                                        <h3 class={classes!(
                                            "text-std-200", "text-xl",
                                            "font-medium", "truncate",
                                            "hidden", "md:inline"
                                        )}>
                                            {position.name()}
                                        </h3>
                                    </div>
                                </th>
                            }
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            {get_recommendations(|position| champion.recommended_items(position))}
                        </tr>
                        <tr>
                            {get_recommendations(|position| champion.recommended_runes(position))}
                        </tr>
                    </tbody>
                </table>
            </div>
            <H2 text={"Source code definition"} />
            <Code fragment={champion.render_global().unwrap()} />
            <H2 text={"Abilities virtual definiton"} />
            // for meta in champion.metadata() {
            //     <Code fragment={champion.render_ability(meta.kind).unwrap()} />
            // }
            <H2 text={"Internal ability functions"} />
            for meta in champion.metadata() {
                <Code fragment={champion.render_fn(meta.kind).unwrap()} />
            }
            <H2 text={"Champion generator implementation"} />
            <Code fragment={champion.render_generator().unwrap()} />
        </div>
    }
}
