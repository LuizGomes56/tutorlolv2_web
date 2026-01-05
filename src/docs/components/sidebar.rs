use yew::prelude::*;

#[component]
pub fn Sidebar() -> Html {
    html! {
        <aside class={classes!(
            "border-r", "border-r-gray-700", "w-72", "block"
        )}>
            <nav class={classes!(
                "h-full", "max-h-screen", "sticky",
                "top-0", "py-2", "pl-2", "pr-4"
            )}>
                <ul class={classes!("list-none")}>
                    {["Champions", "Items", "Runes", "Other"]
                        .into_iter()
                        .map(|text| html! {
                            <li class={classes!(
                                "py-1.5", "px-3", "leading-5", "rounded-md",
                                "transition-all", "hover:bg-std-800", "my-2",
                                "text-std-200"
                            )}>{text}</li>
                        })
                        .collect::<Html>()
                    }
                </ul>
            </nav>
        </aside>
    }
}
