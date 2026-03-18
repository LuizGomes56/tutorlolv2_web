use yew::prelude::*;

#[component]
pub fn HoverDocs() -> Html {
    html! {
        <div id={"hoverdocs"}
            class={classes!(
                "fixed", "inset-auto", "z-50", "pointer-events-auto",
                "hidden", "overflow-auto", "flex-col",
                "box-border",
                "max-w-[calc(100vw-16px)]", "sm:max-w-md",
                "max-h-[calc(100vh-16px)]", "sm:max-h-96",
                "p-2", "leading-6", "text-base", "documentation",
                "border", "border-std-800", "bg-std-900",
            )}>
            <code
                id={"hoverdocs_code"}
                class={classes!(
                    "flex", "flex-col", "gap-2", "text-[#D4D4D4]",
                    "font-normal", "text-left",
                    "break-words"
                )}
            />
        </div>
    }
}
