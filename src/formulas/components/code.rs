use crate::utils::hoverdocs;
use std::ops::Range;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct CodeProps {
    pub range: &'static Range<usize>,
}

#[component]
pub fn Code(props: &CodeProps) -> Html {
    let CodeProps { range } = *props;

    let cache = hoverdocs(range.clone());
    let code = Html::from_html_unchecked(cache.into());

    html! {
        <code class={classes!(
            // "bg-[#1f1f1f]",
            "px-4", "py-3",
            "border", "border-std-800",
            // "max-h-[calc(100vh-16rem)]",
            // "overflow-auto"
        )}>
            {code}
        </code>
    }
}
