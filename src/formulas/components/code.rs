use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct CodeProps {
    pub fragment: AttrValue,
}

#[component]
pub fn Code(props: &CodeProps) -> Html {
    let code = Html::from_html_unchecked(props.fragment.clone());

    html! {
        <code class={classes!(
            // "bg-[#1f1f1f]",
            "px-4", "py-3",
            "border", "border-std-800",
            // "max-h-[calc(100vh-16rem)]",
            "overflow-auto"
        )}>
            <pre>{code}</pre>
        </code>
    }
}
