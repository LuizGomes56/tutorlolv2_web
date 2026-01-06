use crate::Route;
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
                            "py-2", "px-4", "leading-5", "rounded-md",
                            "transition-all", "hover:bg-std-800",
                            "text-std-200", "text-lg"
                        )}
                    >
                        {text}
                    </Link<Route>>
                }
            })
            .collect::<Html>()
    }

    html! {
        <aside class={classes!(
            "w-64", "top-0", "fixed", "left-0",
            "h-full", "max-h-screen", "pt-[60px]"
        )}>
            <nav class={classes!(
                "flex", "flex-col", "h-full",
                "py-4", "pl-2", "pr-4", "justify-between",
                "border-r", "border-r-gray-700",
            )}>
                <div class={classes!("flex", "flex-col", "gap-2")}>
                    {buttons([
                        ("Homepage", Route::Homepage),
                        ("Calculator", Route::Calculator),
                        ("Livegame", Route::Livegame)
                    ])}
                </div>
                <div class={classes!("flex", "flex-col", "gap-2")}>
                    {buttons([
                        ("Documentation", Route::Docs),
                        ("Help", Route::Docs),
                        ("FAQ", Route::FAQ),
                        ("About", Route::About),
                        ("GitHub", Route::Homepage)
                    ])}
                </div>
            </nav>
        </aside>
    }
}
