use crate::{components::image::Image, utils::EnumCast};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct TrayProps<T: PartialEq> {
    pub vector: Vec<T>,
    pub callback: Callback<usize>,
}

#[component]
pub fn Tray<T: EnumCast>(props: &TrayProps<T>) -> Html {
    let TrayProps { vector, callback } = props;

    let values = vector
        .iter()
        .enumerate()
        .map(|(i, value)| {
            html! {
                <button
                    class={classes!("flex", "items-center", "gap-2")}
                    onclick={{
                        let callback = callback.clone();
                        Callback::from(move |_| {
                            callback.emit(i);
                        })
                    }}
                >
                    <Image src={value.image_type()} class={classes!("w-6", "h-6")} />
                    <span>{value.name()}</span>
                </button>
            }
        })
        .collect::<Html>();

    html! {
        <div>{values}</div>
    }
}
