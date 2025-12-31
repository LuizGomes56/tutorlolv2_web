use crate::utils::ImageType;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ImageProps {
    #[prop_or(classes!("w-6", "h-6", "opacity-50"))]
    pub class: Classes,
    pub src: ImageType,
}

#[component]
pub fn Image(props: &ImageProps) -> Html {
    let ImageProps { class, src } = props;
    let (main_offset, exc_offset) = src.offset();
    let header = src.header();
    let src = src.url();

    let mut classes = classes!("relative");
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
