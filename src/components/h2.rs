use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct H2Props {
    pub text: AttrValue,
}

#[component]
pub fn H2(props: &H2Props) -> Html {
    let H2Props { text } = props;

    html! {
        // <div class={classes!("flex", "items-center", "gap-8")}>
            <h2 class={classes!(
                "text-2xl", "text-std-200", "font-medium"
            )}>
                {text}
            </h2>
            // <hr class={classes!("flex-1", "border", "border-std-500")} />
        // </div>
    }
}
