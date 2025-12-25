use crate::{calculator::reducer::DragonAction, model::Dragons};
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct DragonProps {
    dragons: UseReducerHandle<Dragons>,
}

#[component]
pub fn Dragon(props: &DragonProps) -> Html {
    let DragonProps { dragons } = props;

    html! {
        <div></div>
    }
}
