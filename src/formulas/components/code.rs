use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct CodeProps {
    pub value: Option<AttrValue>,
}

#[component]
pub fn Code(props: &CodeProps) -> Html {
    let CodeProps { value } = props;

    match value {
        Some(v) => {
            let code = Html::from_html_unchecked(v.into());

            html! {
                <code class={classes!(
                    // "bg-[#1f1f1f]",
                    "px-4", "py-3",
                    "border", "border-std-800",
                    // "max-h-[calc(100vh-16rem)]",
                    "overflow-auto"
                )}>
                    {code}
                </code>
            }
        }
        None => Default::default(),
    }
}
