use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct CheckboxProps {
    pub callback: Callback<bool>,
    pub checked: bool,
    pub label: AttrValue,
}

#[component]
pub fn Checkbox(props: &CheckboxProps) -> Html {
    let CheckboxProps {
        checked,
        ref callback,
        ref label,
    } = *props;

    let onchange = use_callback(callback.clone(), |e: Event, callback| {
        let target = e.target_unchecked_into::<HtmlInputElement>();
        callback.emit(target.checked());
    });

    html! {
        <label class={classes!(
            "inline-flex",
            "items-center",
            "gap-2",
            "cursor-pointer",
            "select-none",
            "px-4",
            "py-2",
            "transition-colors",
            "hover:bg-std-800/60"
        )}>
            <input
                type={"checkbox"}
                class={classes!("peer", "sr-only")}
                {checked}
                {onchange}
            />
            <span class={classes!(
                "flex",
                "h-3.5",
                "w-3.5",
                "items-center",
                "justify-center",
                "border",
                "border-std-600",
                "bg-std-900",
                "text-white",
                "transition-all",
                "duration-150",
                "peer-checked:border-violet-500",
                "peer-checked:bg-violet-600",
                "peer-focus-visible:outline",
                "peer-focus-visible:outline-2",
                "peer-focus-visible:outline-violet-400"
            )}>
                if checked {
                    <svg
                        xmlns={"http://www.w3.org/2000/svg"}
                        class={"h-3.5 w-3.5"}
                        viewBox={"0 0 20 20"}
                        fill={"currentColor"}
                    >
                        <path
                            fill-rule={"evenodd"}
                            d={"M16.704 5.29a1 1 0 010 1.414l-7.2 7.2a1 1 0 01-1.414 0l-3-3a1 1 0 111.414-1.414l2.293 2.293 6.493-6.493a1 1 0 011.414 0z"}
                            clip-rule={"evenodd"}
                        />
                    </svg>
                }
            </span>
            <span class={classes!(
                "text-std-200",
                "transition-colors",
                "peer-checked:text-white",
                "text-sm"
            )}>
                {label}
            </span>
        </label>
    }
}
