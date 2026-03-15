use crate::{
    calculator::reducer::LastAction,
    components::image::{DragonImage, Image, ImageType, OtherImage},
    model::{Dragons, DragonsAction},
};
use std::{cell::RefCell, rc::Rc};
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[hook]
pub fn use_dragons(
    dragons: &UseReducerHandle<Dragons>,
    last_action: &Rc<RefCell<LastAction>>,
    f: fn(u16) -> DragonsAction,
) -> Callback<InputEvent> {
    let dragons = dragons.clone();
    let last_action = last_action.clone();
    use_callback((), move |e: InputEvent, _| {
        let target = e.target_unchecked_into::<HtmlInputElement>();
        let value = target.value().parse::<u16>().unwrap_or(0);
        last_action.replace(LastAction::CurrentPlayer);
        dragons.dispatch(f(value));
    })
}

#[derive(PartialEq, Properties)]
pub struct DragonInputProps {
    pub title: AttrValue,
    pub oninput: Callback<InputEvent>,
    pub src: DragonImage,
    pub value: u16,
}

#[component]
pub fn DragonInput(props: &DragonInputProps) -> Html {
    let DragonInputProps {
        ref title,
        ref oninput,
        src,
        value,
    } = *props;

    html! {
        <>
            <Image
                class={classes!("w-6", "h-6")}
                src={ImageType::Other(OtherImage::Dragon(src))}
            />
            <span class={classes!("text-sm", "content-center", "truncate")}>
                {title}
            </span>
            <input
                type={"number"}
                class={classes!(
                    "text-center", "min-w-0", "ml-2",
                    "bg-transparent", "text-white"
                )}
                value={value.to_string()}
                oninput={oninput.clone()}
            />
        </>
    }
}
