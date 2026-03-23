use crate::{Route, components::image::Svg};
use yew::prelude::*;
use yew_router::components::Link;

#[component]
pub fn Sidebar() -> Html {
    fn buttons<const N: usize>(array: [(&str, Route); N]) -> Html {
        array
            .into_iter()
            .map(|(text, to)| {
                html! {
                    <Link<Route>
                        to={to}
                        classes={classes!(
                            "py-3", "px-4", "leading-5", "rounded-md",
                            "transition-all", "hover:bg-std-800",
                            "text-std-200", "flex", "items-center",
                            "gap-4"
                        )}
                    >
                        <Svg
                            class={classes!("h-5", "w-5")}
                            src={format!("/svgs/sidebar/{text}.svg")}
                        />
                        <span class={classes!("text-shadow", "font-medium")}>
                            {text}
                        </span>
                    </Link<Route>>
                }
            })
            .collect::<Html>()
    }

    html! {
        <aside class={classes!(
            "w-48", "h-full", "max-h-screen",
            "hidden", "md:block"
        )}>
            <nav class={classes!(
                "flex", "flex-col", "h-full",
                "py-4", "px-4", "justify-between",
                "box",
            )}>
                <div class={classes!("flex", "flex-col", "gap-2")}>
                    {buttons([
                        ("Homepage",    Route::Homepage),
                        ("Calculator",  Route::Calculator),
                        ("Livegame",    Route::Livegame)
                    ])}
                </div>
                <div class={classes!("flex", "flex-col", "gap-2")}>
                    {buttons([
                        ("Formulas",    Route::Formulas),
                        ("Help & FAQ",    Route::Formulas),
                        ("About",   Route::About),
                        ("GitHub",  Route::Homepage)
                    ])}
                </div>
            </nav>
        </aside>
    }
}
