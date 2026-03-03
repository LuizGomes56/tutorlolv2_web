use crate::{
    components::image::Image,
    utils::{EnumCast, encode_offset},
};
use std::cmp::Ordering;
use tutorlolv2_gen::ItemId;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct SelectorProps<T: PartialEq> {
    pub callback: Callback<T>,
    #[prop_or_else(|| Callback::from(|_: T| true))]
    pub filter: Callback<T, bool>,
    #[prop_or_default]
    pub sort_by: Option<Callback<(T, T), Ordering>>,
}

#[component]
pub fn Selector<T: EnumCast>(props: &SelectorProps<T>) -> Html {
    let SelectorProps {
        callback,
        filter,
        sort_by,
    } = props;

    let items = use_memo(
        (callback.clone(), filter.clone(), sort_by.clone()),
        |(callback, filter, sort_by)| {
            let mut data = T::VALUES
                .iter()
                .filter(|&&value| filter.emit(value))
                .collect::<Vec<_>>();

            if let Some(sort_by) = sort_by {
                data.sort_by(|&&a, &&b| sort_by.emit((a, b)));
            }

            data.into_iter()
                .map(|value| {
                    let onclick = {
                        let callback = callback.clone();
                        Callback::from(move |_| {
                            callback.emit(*value);
                        })
                    };

                    let data_offset = encode_offset(&[value.formula()]);

                    html! {
                        <button
                            {data_offset}
                            {onclick}
                        >
                            <div class={classes!("flex", "items-center", "gap-2")}>
                                <Image src={value.image_type()} class={classes!("w-6", "h-6")} />
                                <span class={classes!("truncate")}>{value.name()}</span>
                            </div>
                        </button>
                    }
                })
                .collect::<Html>()
        },
    );

    html! {
        <div class={classes!(
            "flex", "flex-col", "max-h-48",
            "overflow-auto", "gap-2"
        )}>
            {(*items).clone()}
        </div>
    }
}

pub const fn item_filter(item: ItemId) -> bool {
    let cache = item.cache();
    cache.maps.summoners_rift
        && cache.purchasable
        && !cache.prettified_stats.is_empty()
        && cache.riot_id < 100000
}
