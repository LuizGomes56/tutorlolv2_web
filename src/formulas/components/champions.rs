use crate::{
    components::{
        image::{Image, ImageType},
        selector::Selector,
    },
    formulas::components::{Section, code::Code},
    utils::{EnumCast, encode_offset, use_setter},
};
use tutorlolv2_gen::{CastId, ChampionId, Position};
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
                    <div class={classes!("flex", "flex-col", "gap-2")}>
                        for item in f(position) {
                            <div data_offset={encode_offset(&[item.formula()])}>
                                <div class={classes!("flex", "items-center", "gap-3")}>
                                    <Image
                                        class={classes!("w-7", "h-7")}
                                        src={ImageType::from(*item)}
                                    />
                                    <span class={classes!(
                                        "text-std-300", "truncate"
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
        <div class={classes!("flex", "flex-col", "gap-6", "py-4", "px-6", "box")}>
            <Section text={"Champions"} />
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
            <Section text={"Recommended items and runes per position"} />
            <div class={classes!("overflow-auto")}>
                <table class={classes!("table-fixed")}>
                    <thead>
                        <tr>
                            for position in Position::ARRAY {
                                <th>
                                    <div class={classes!("flex", "items-center", "gap-3")}>
                                        <Image
                                            class={classes!("w-8", "h-8")}
                                            src={ImageType::Position(position)}
                                        />
                                        <h3 class={classes!(
                                            "text-std-200", "text-xl",
                                            "font-medium", "truncate"
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
            <Section text={"Source code definition"} />
            <Code range={champion.formula()} />
            <Section text={"Abilities virtual definiton"} />
            for i in 0..champion.number_of_abilities() {
                <Code range={champion.get_ability_formula(i)} />
            }
            <Section text={"Internal ability functions"} />
            for i in 0..champion.number_of_abilities() {
                <Code range={&champion.closures()[i]} />
            }
            <Section text={"Champion generator implementation"} />
            <Code range={champion.generator()} />
        </div>
    }
}
