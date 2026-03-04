use crate::{
    calculator::ExceptionMap,
    components::image::{Image, ImageType},
    utils::encode_offset,
};
use std::hash::Hash;
use tutorlolv2_gen::{CastId, ChampionId};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct ExceptionInputProps<T: PartialEq> {
    pub oninput: Callback<InputEvent>,
    #[prop_or(None)]
    pub value: Option<T>,
    pub image_type: ImageType,
    pub stacks: u32,
}

#[component]
pub fn ExceptionInput<T>(props: &ExceptionInputProps<T>) -> Html
where
    T: CastId + PartialEq,
{
    let ExceptionInputProps {
        ref oninput,
        value,
        image_type,
        stacks,
    } = *props;

    let data_offset = value.map(|v| encode_offset(&[v.formula()]));
    let title = value.map(|v| v.name());

    html! {
        <div class={classes!("flex", "items-center", "gap-2")}>
            <div {title} {data_offset}>
                <Image
                    class={classes!("w-8", "h-8")}
                    src={image_type}
                />
            </div>
            <input
                type={"number"}
                class={classes!(
                    "text-center", "min-w-0", "ml-2",
                    "bg-transparent", "text-white"
                )}
                {oninput}
                value={stacks.to_string()}
                placeholder={"0"}
            />
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ExceptionSelectorProps<T: Default + Eq + Hash + 'static> {
    pub values: Vec<T>,
    pub exceptions: ExceptionMap<T>,
    pub callback: Callback<(T, u32)>,
    pub filter: &'static [T],
}

#[component]
pub fn ExceptionSelector<T>(props: &ExceptionSelectorProps<T>) -> Html
where
    T: CastId + Default + Eq + Hash,
    ImageType: From<T>,
{
    let ExceptionSelectorProps {
        ref values,
        ref exceptions,
        ref callback,
        filter,
    } = *props;

    values
        .iter()
        .filter(|value| filter.contains(value))
        .filter_map(|&value| {
            exceptions.inner.get(&value).map(|v| {
                let oninput = {
                    let callback = callback.clone();
                    Callback::from(move |e: InputEvent| {
                        let target = e.target_unchecked_into::<HtmlInputElement>();
                        let stacks = target.value().parse::<u32>().unwrap_or(0);
                        callback.emit((value, stacks));
                    })
                };

                html!(
                    <ExceptionInput<T>
                        {oninput}
                        {value}
                        image_type={ImageType::from(value)}
                        stacks={v.stacks()}
                    />
                )
            })
        })
        .collect::<Html>()
}

#[derive(PartialEq, Properties)]
pub struct ChampionExceptionSelectorProps {
    pub callback: Callback<u32>,
    pub stacks: u32,
    pub champion_id: ChampionId,
    #[prop_or(false)]
    pub ally: bool,
}

#[component]
pub fn ChampionExceptionSelector(props: &ChampionExceptionSelectorProps) -> Html {
    let ChampionExceptionSelectorProps {
        ref callback,
        stacks,
        champion_id,
        ally,
    } = *props;

    let oninput = use_callback(callback.clone(), move |e: InputEvent, callback| {
        let target = e.target_unchecked_into::<HtmlInputElement>();
        let value = target.value().parse::<u32>().unwrap_or(0);
        callback.emit(value);
    });

    champion_id
        .exceptions(ally)
        .map(|key| {
            html!(
                <ExceptionInput<ChampionId>
                    {oninput}
                    image_type={ImageType::Ability(champion_id, key.into())}
                    {stacks}
                />
            )
        })
        .unwrap_or_default()
}
