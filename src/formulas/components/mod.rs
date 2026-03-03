pub mod champions;
pub mod code;
pub mod items;
pub mod runes;

#[derive(PartialEq, Properties)]
pub struct SectionProps {
    pub text: AttrValue,
}

use yew::prelude::*;

#[component]
pub fn Section(props: &SectionProps) -> Html {
    let SectionProps { text } = props;

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
