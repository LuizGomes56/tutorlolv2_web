use crate::{
    components::image::{Image, ImageType, Svg},
    utils::{EnumCast, hooks::use_clickout},
};
use tutorlolv2_gen::{CastId, ItemId};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct SelectorProps<T: CastId + PartialEq + 'static> {
    pub callback: Callback<T>,
    pub value: T,
    #[prop_or(classes!("gap-4"))]
    pub label_class: Classes,
    #[prop_or(classes!("w-12", "h-12"))]
    pub img_class: Classes,
    #[prop_or(classes!(
        "text-std-200", "placeholder:text-std-500",
        "text-3xl", "font-medium", "w-full",
    ))]
    pub input_class: Classes,
    #[prop_or(classes!("mt-2", "px-1.5", "py-1", "w-96"))]
    pub dropdown_class: Classes,
    #[prop_or(T::VALUES)]
    pub array: &'static [T],
}

#[component]
pub fn Selector<T>(props: &SelectorProps<T>) -> Html
where
    T: EnumCast,
    ImageType: From<T>,
{
    let SelectorProps {
        ref callback,
        ref img_class,
        ref input_class,
        ref label_class,
        ref dropdown_class,
        array,
        value,
    } = *props;

    let is_open = use_state(|| false);
    let query = use_state(String::new);

    let close_callback = use_callback(is_open.clone(), move |_, is_open| is_open.set(false));

    let dropdown_ref = use_node_ref();
    let label_ref = use_clickout(close_callback.clone(), [dropdown_ref.clone()]);

    let buttons =
        use_memo(
            (callback.clone(), close_callback.clone(), query.clone()),
            |(callback, close_callback, query)| {
                array
                .iter()
                .map(|&v| {
                    let onclick = {
                        let callback = callback.clone();
                        let close_callback = close_callback.clone();
                        let query = query.clone();
                        Callback::from(move |_| {
                            callback.emit(v);
                            close_callback.emit(());
                            query.set(String::new());
                        })
                    };

                    (v, html! {
                        <button key={v.index()} {onclick}>
                            <div class={classes!("flex", "items-center", "gap-2", "py-0.5")}>
                                <Image src={v.image_type()} class={classes!("w-6", "h-6")} />
                                <span class={classes!("truncate")}>{v.name()}</span>
                            </div>
                        </button>
                    })
                })
                .collect::<Box<[(T, Html)]>>()
            },
        );

    let options = buttons
        .iter()
        .filter(|(v, _)| {
            query.is_empty()
                || v.name()
                    .to_ascii_lowercase()
                    .contains(&query.trim().to_ascii_lowercase())
        })
        .map(|(_, v)| v.clone())
        .collect::<Html>();

    let oninput = use_callback(
        (query.clone(), is_open.clone()),
        |e: InputEvent, (query, is_open)| {
            let target = e.target_unchecked_into::<HtmlInputElement>();
            let value = target.value();
            is_open.set(!value.is_empty());
            query.set(value);
        },
    );

    let onfocus = use_callback(is_open.clone(), move |_: FocusEvent, is_open| {
        is_open.set(true)
    });

    html! {
        <div class={classes!("relative")}>
            <label ref={label_ref} class={classes!(
                "h-full", "block", "content-end"
            )}>
                <div class={{
                    let mut class = classes!("flex", "items-center");
                    class.push(label_class);
                    class
                }}>
                    <Image
                        class={img_class}
                        src={ImageType::from(value)}
                    />
                    <input
                        class={{
                            let mut class = classes!(
                                "bg-transparent", "focus:ring-0",
                                "focus:outline-none", "truncate"
                            );
                            class.push(input_class);
                            class
                        }}
                        value={query.to_string()}
                        placeholder={value.name()}
                        {oninput}
                        {onfocus}
                    />
                </div>
            </label>
            <div
                ref={dropdown_ref}
                class={{
                    let mut class = classes!(
                        match *is_open {
                            true => "flex",
                            false => "hidden",
                        },
                        "absolute",
                        "bg-[#16161c]", "flex-col",
                        "overflow-auto", "max-h-72",
                        "border", "border-std-800",
                        "z-50", "empty:hidden"
                    );
                    class.push(dropdown_class);
                    class
                }}
            >
                {options}
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct SelectorButtonProps {
    pub title: AttrValue,
    pub onclick: Callback<MouseEvent>,
    pub length: usize,
}

#[component]
pub fn SelectorButton(props: &SelectorButtonProps) -> Html {
    let SelectorButtonProps {
        ref title,
        ref onclick,
        length,
    } = *props;

    html! {
        <button {onclick} class={classes!(
            "transition-all", "duration-150",
            "hover:bg-std-800/60",
            "hover:border-std-700",
            "border-y", "border-std-800",
            "p-2", "flex", "items-center", "justify-between",
            "gap-3", "group", "w-full"
        )}>
            <div class={classes!("flex", "items-center", "gap-3", "pl-2")}>
                <div class={classes!("text-sm", "font-medium", "text-std-100")}>
                    {title}
                </div>
                <div class={classes!(
                    "text-xs", "text-std-400"
                )}>
                    {"Add / Remove"}
                </div>
            </div>
            <div class={classes!("flex", "items-center", "gap-2", "shrink-0")}>
                <span class={classes!(
                    "min-w-[2rem]",
                    "px-1.5", "py-0.5",
                    "bg-sky-500/15",
                    "border", "border-sky-500/20",
                    "text-sky-300",
                    "text-sm", "font-semibold", "font-mono",
                    "text-center"
                )}>
                    {length}
                </span>
                <Svg
                    class={classes!(
                        "h-4", "w-4", "text-std-500", "transition-transform",
                        "duration-150", "group-hover:text-std-300"
                    )}
                    src={"/svgs/rchev.svg"}
                />
            </div>
        </button>
    }
}

pub const fn item_filter(item: ItemId) -> bool {
    let cache = item.cache();
    cache.maps.summoners_rift
        && cache.purchasable
        && !cache.prettified_stats.is_empty()
        && cache.riot_id < 100000
}
