use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct EmptyTableProps {
    pub rows: usize,
}

#[component]
pub fn EmptyTable(props: &EmptyTableProps) -> Html {
    let EmptyTableProps { rows } = *props;
    html! {
        <div class={classes!("box")}>
            <table>
                <thead><tr><th></th></tr></thead>
                <tbody>
                    for _ in 0..rows { <tr><td></td></tr> }
                </tbody>
            </table>
        </div>
    }
}
