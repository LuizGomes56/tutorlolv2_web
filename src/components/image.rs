use crate::utils::ImageType;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ImageProps {
    #[prop_or(classes!("w-8", "h-8"))]
    pub class: Classes,
    pub src: ImageType,
}

#[component]
pub fn Image(props: &ImageProps) -> Html {
    let ImageProps { class, src } = props;
    let (main_offset, exc_offset) = src.offset();
    let header = src.header();
    let src = src.url();

    let mut classes = classes!("flex", "items-center", "justify-center", "relative", "cell");
    classes.push(class);

    html! {
        <div
            data-offset-main={main_offset}
            data-offset-exc={exc_offset}
            class={classes}
        >
            <img loading={"lazy"} {src} alt={""} />
            {header}
        </div>
    }
}
