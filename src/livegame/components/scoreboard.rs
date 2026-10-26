use crate::livegame::Scoreboard;
use std::rc::Rc;
use strum::VariantArray;
use tutorlolv2::{Position, model::Team};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ScoreboardProps {
    pub scoreboard: Rc<[Scoreboard]>,
    pub ally_team: Team,
}

fn team_by_position(
    scoreboard: &[Scoreboard],
    f: impl Fn(&Scoreboard) -> bool + Copy,
) -> impl Iterator<Item = &Scoreboard> {
    Position::VARIANTS.iter().flat_map(move |&position| {
        scoreboard
            .iter()
            .filter(move |s| s.position == position && f(s))
    })
}

#[component]
pub fn ScoreboardDisplay(props: &ScoreboardProps) -> Html {
    let ScoreboardProps {
        ref scoreboard,
        ally_team,
    } = *props;

    let allies = team_by_position(scoreboard, |s| s.team == ally_team);
    let enemies = team_by_position(scoreboard, |s| s.team != ally_team);

    html! {
        <>
            <div class={classes!("box", "overflow-auto")}>
                <table>
                    <thead>
                        <tr>
                            <th>{"Allies"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        for ally in allies {
                            <tr>{ally}</tr>
                        }
                    </tbody>
                </table>
            </div>
            <div class={classes!("box", "overflow-auto")}>
                <table>
                    <thead>
                        <tr>
                            <th>{"Enemies"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        for enemy in enemies {
                            <tr>{enemy}</tr>
                        }
                    </tbody>
                </table>
            </div>
        </>
    }
}
