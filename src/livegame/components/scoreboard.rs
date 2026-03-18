use crate::{
    components::image::{Image, ImageType},
    livegame::Scoreboard,
    model::Team,
};
use std::rc::Rc;
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

fn render_cell(entry: Option<&Scoreboard>) -> Html {
    let Some(entry) = entry else {
        return Default::default();
    };

    let Scoreboard {
        riot_id,
        assists,
        creep_score,
        deaths,
        kills,
        champion_id,
        position,
        ..
    } = entry;

    html! {
        <div class={classes!(
            "grid", "grid-cols-[auto_1fr_auto]",
            "gap-2", "items-center"
        )}>
            <div class={classes!("relative", "shrink-0")}>
                <Image
                    class={classes!("w-8", "h-8", "overflow-hidden")}
                    src={ImageType::from(champion_id)}
                />
            </div>
            <div class={classes!(
                "flex", "min-w-0", "flex-col", "gap-0.5",
            )}>
                <span class={classes!(
                    "text-left",
                    "text-xs",
                    "text-std-100",
                    "truncate"
                )}>
                    {riot_id.split_once('#').map(|(left, _)| left).unwrap_or(riot_id)}
                </span>
                <div class={classes!(
                    "flex", "items-center", "gap-1.5", "min-w-0"
                )}>
                    <Image
                        class={classes!("w-3.5", "h-3.5", "shrink-0")}
                        src={ImageType::Position(*position)}
                    />
                    <span class={classes!(
                        "truncate",
                        "text-xs",
                        "text-std-400"
                    )}>
                        {champion_id.name()}
                    </span>
                    <span class={classes!(
                        "shrink-0",
                        "text-xs",
                        "text-std-400"
                    )}>
                        {"(CS: "}{creep_score}{")"}
                    </span>
                </div>
            </div>
            <div class={classes!(
                "shrink-0",
                "px-2",
                "text-sm",
                "text-std-200",
                "whitespace-nowrap"
            )}>
                {kills}
                <span class={classes!("text-std-400")}>{" / "}</span>
                {deaths}
                <span class={classes!("text-std-400")}>{" / "}</span>
                {assists}
            </div>
        </div>
    }
}

#[component]
pub fn ScoreboardDisplay(props: &ScoreboardProps) -> Html {
    let ScoreboardProps {
        ref scoreboard,
        ally_team,
    } = *props;

    let allies = sort_team(scoreboard, |s| s.team == ally_team);
    let enemies = sort_team(scoreboard, |s| s.team != ally_team);

    let row_count = allies.len().max(enemies.len());

    html! {
        <div class={classes!("box", "overflow-auto")}>
            <table>
                <thead>
                    <tr>
                        <th>{"Allies"}</th>
                        <th>{"Enemies"}</th>
                    </tr>
                </thead>
                <tbody>
                    for i in 0..row_count {
                        <tr>
                            <td>
                                {render_cell(allies.get(i).copied())}
                            </td>
                            <td>
                                {render_cell(enemies.get(i).copied())}
                            </td>
                        </tr>
                    }
                </tbody>
            </table>
        </div>
    }
}
