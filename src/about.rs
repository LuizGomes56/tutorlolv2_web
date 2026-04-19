use crate::components::h2::H2;
use yew::prelude::*;

#[component]
pub fn About() -> Html {
    html! {
        <div class={classes!("w-full", "h-full", "p-4")}>
            <div class={classes!("flex", "flex-col", "h-full", "gap-6", "p-6", "box")}>
                <H2 text={"About this project"} />
                <p class={classes!("text-std-400")}>
                    {concat!(
                        "This is project was done independently, without any connections ",
                        "or endorsement from Riot Games, and all the data being used in the ",
                        "live game section is provided by Riot Games API, and other supplemental ",
                        "information comes from manual implementation and public APIs such as ",
                        "Meraki Analytics. In addition this project is open source and you can ",
                        "find the source code by going to my GitHub repository or holding SHIFT ",
                        "while hovering you mouse over some objects, which will show a simplified ",
                        "representation of the source code being used in the current version of ",
                        "this app"
                    )}
                </p>
                <H2 text={"Timeline"} />
                <p class={classes!("text-std-400")}>
                    {concat!(
                        "The initial idea started on February 2023 and was first released on ",
                        "July 2023 as a prototype. However, manually updating the app was tedious ",
                        "and time-consuming, also the calculations were limited to only 5 abilities ",
                        "due to the API initial design. This new version comes to resolve these issues ",
                        "by making calculations faster, performing automatic updates (although they need ",
                        "to be recompiled every time a new patch is released), and allowing a champion to ",
                        "have as many abilities as needed"
                    )}
                </p>
                <p class={classes!("text-std-400")}>
                    {concat!(
                        "The new version begun its development on May 10, 2025 and was concluded on ",
                        "March 21, 2026"
                    )}
                </p>
                <H2 text={"Notes"} />
                <p class={classes!("text-std-400")}>
                    {concat!(
                        "Riot's API sometimes have breaking changes which may cause this app to crash ",
                        "or show incorrect results. There's no guarantee that this app will be updated ",
                        "after every new patch release, so if there's no update I made available, the ",
                        "only for you to force an update is to download the source code and compile it ",
                        "in your own machine"
                    )}
                </p>
            </div>
        </div>
    }
}
