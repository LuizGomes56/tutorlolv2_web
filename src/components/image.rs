use crate::utils::ImageType;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ImageProps {
    #[prop_or_default]
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
            data-offset-main={main_offset.as_ref().map(ToString::to_string)}
            data-offset-exc={exc_offset.as_ref().map(ToString::to_string)}
            class={classes}
        >
            <img loading={"lazy"} {src} alt={""} />
            {header}
        </div>
    }
}
