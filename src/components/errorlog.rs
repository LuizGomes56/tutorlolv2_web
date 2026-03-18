use crate::utils::Loading;
use yew::prelude::*;

pub fn errorlog<T: AsRef<dyn core::error::Error>>(error: &T) -> Html {
    let e = error.as_ref();

    html! {
        if !e.is::<Loading>() {
            <div class={classes!("box")}>
                <div class={classes!(
                    "grid", "md:grid-cols-2", "gap-6",
                    "px-6", "py-4", "bg-std-900"
                )}>
                    <div class={classes!("flex", "flex-col", "gap-4")}>
                        <h2 class={classes!("text-2xl", "text-std-200", "font-medium")}>
                            {"Request error"}
                        </h2>
                        <ul class={classes!("text-std-400", "ml-8")}>
                            <li class={classes!("list-disc")}>
                                {"Servers might be down due to an internal error"}
                            </li>
                            <li class={classes!("list-disc")}>
                                {"This application might be outdated"}
                            </li>
                            <li class={classes!("list-disc")}>
                                {"Refresh the page or come back later"}
                            </li>
                        </ul>
                    </div>
                    <code class={classes!(
                        "flex", "flex-col",
                        "overflow-auto", "p-2",
                        "leading-6", "text-base", "border",
                        "border-std-800", "bg-std-900"
                    )}>
                        <pre class={classes!("whitespace-pre-wrap")}>
                            {format!("{e:#?}")}
                        </pre>
                    </code>
                </div>
            </div>
        }
    }
}
