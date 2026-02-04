use crate::{components::image::Image, utils::EnumCast};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct SelectorProps<T: PartialEq> {
    pub callback: Callback<T>,
}

#[component]
pub fn Selector<T: EnumCast>(props: &SelectorProps<T>) -> Html {
    let SelectorProps { callback } = props;

    let items = use_memo(callback.clone(), |callback| {
        T::ARRAY
            .iter()
            .map(|item| {
                let onclick = {
                    let callback = callback.clone();
                    Callback::from(move |_| {
                        callback.emit(*item);
                    })
                };

                html! {
                    <button {onclick} class={classes!("flex", "items-center", "gap-2")}>
                        <Image src={item.image_type()} class={classes!("w-6", "h-6")} />
                        <span class={classes!("truncate")}>{item.name()}</span>
                    </button>
                }
            })
            .collect::<Html>()
    });

    html! {
        <div class={classes!(
            "flex", "flex-col", "max-h-48",
            "overflow-auto", "gap-2"
        )}>
            {(*items).clone()}
        </div>
    }
}
