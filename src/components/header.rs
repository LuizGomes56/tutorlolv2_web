use yew::prelude::*;

#[derive(PartialEq, Properties)]
struct HeaderButtonProps<const N: usize> {
    array: [&'static str; N],
}

#[component]
fn HeaderButton<const N: usize>(props: &HeaderButtonProps<N>) -> Html {
    let HeaderButtonProps { array } = props;
    html! {
        array
            .into_iter()
            .map(|&text| {
                html! {
                    <a class={classes!(
                        "inline-block", "hover:text-emerald-400",
                        "py-1", "px-3", "font-medium", "transition-colors"
                    )}>{text}</a>
                }
            })
            .collect::<Html>()
    }
}

#[component]
pub fn Header() -> Html {
    html! {
        <nav class={classes!(
            "bg-neutral-800", "flex", "py-2",
            "px-4", "sticky", "z-50", "top-0"
        )}>
            <div class={classes!(
                "flex", "justify-between", "items-center",
                "flex-wrap", "w-full", "py-1.5"
            )}>
                <div class={classes!("flex", "flex-1", "items-center")}>
                    <HeaderButton<3> array={["Tutorlolv2", "Livegame", "Calculator"]} />
                </div>
                <div class={classes!("flex-none", "justify-end")}>
                    <HeaderButton<3> array={["Help", "Docs", "GitHub"]} />
                </div>
            </div>
        </nav>
    }
}
