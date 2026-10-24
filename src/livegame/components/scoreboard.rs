use crate::livegame::Scoreboard;
use std::rc::Rc;
use tutorlolv2::model::Team;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ScoreboardProps {
    pub scoreboard: Rc<[Scoreboard]>,
    pub ally_team: Team,
}

fn sort_team(scoreboard: &[Scoreboard], f: impl Fn(&Scoreboard) -> bool) -> Vec<&Scoreboard> {
    let mut data = scoreboard.iter().filter(|s| f(s)).collect::<Vec<_>>();
    data.sort_by_key(|s| s.position.index());
    data
}

#[component]
pub fn ScoreboardDisplay(props: &ScoreboardProps) -> Html {
    let ScoreboardProps {
        ref scoreboard,
        ally_team,
    } = *props;

    let allies = sort_team(scoreboard, |s| s.team == ally_team);
    let enemies = sort_team(scoreboard, |s| s.team != ally_team);

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
                            <tr>
                                <td>
                                    {ally.to_html()}
                                </td>
                            </tr>
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
                            <tr>
                                <td>
                                    {enemy.to_html()}
                                </td>
                            </tr>
                        }
                    </tbody>
                </table>
            </div>
        </>
    }
}
