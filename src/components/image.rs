use crate::utils::ImageType;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ImageProps {
    #[prop_or(classes!("w-10", "h-10"))]
    pub class: Classes,
    pub src: ImageType,
}

#[component]
pub fn Image(props: &ImageProps) -> Html {
    let ImageProps { class, src } = props;
    let (main_offset, exc_offset) = src.offset();
    let header = src.header();
    let src = src.url();

    html! {
        <div
            {class}
            data-offset-main={main_offset}
            data-offset-exc={exc_offset}
        >
            <img {src} />
            {header}
        </div>
    }
}
