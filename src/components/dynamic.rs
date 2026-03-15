use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct DynamicProps {
    pub children: Children,
    pub panel_id: AttrValue,
    pub focused: bool,
}

#[component]
pub fn Dynamic(props: &DynamicProps) -> Html {
    let DynamicProps {
        children,
        panel_id,
        focused,
    } = props;

    let circle = |handle, class| {
        let mut base = classes!(
            "absolute",
            "w-2.5",
            "h-2.5",
            "rounded-full",
            "bg-std-700",
            "z-10",
        );

        base.push(class);

        html!(<div data-resize-handle={handle} class={base} />)
    };

    html! {
        <div
            data-panel-id={panel_id}
            data-panel={true}
            data-scale={1}
            data-active={false}
            class={classes!(
                "border", "border-2", "place-self-end",
                "border-dashed", "select-none",
                "touch-none", "p-2",
                if *focused {
                    classes!("border-std-700", "cursor-move", "bg-std-900/75")
                } else {
                    classes!("border-transparent", "cursor-auto", "bg-transparent")
                }
            )}
        >
            {children}
            {circle("nw", classes!("-left-1.5", "-top-1.5", "cursor-nwse-resize"))}
            {circle("ne", classes!("-right-1.5", "-top-1.5", "cursor-nesw-resize"))}
            {circle("sw", classes!("-left-1.5", "-bottom-1.5", "cursor-nesw-resize"))}
            {circle("se", classes!("-right-1.5", "-bottom-1.5", "cursor-nwse-resize"))}
        </div>
    }
}
