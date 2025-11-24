use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FabProps {
    pub children: Html,
}

#[function_component(Fab)]
pub fn fab(props: &FabProps) -> Html {
    html! {
        <div style="position: fixed; bottom: 0em; right: 0em; padding: 1em; z-index: 100; display: flex; gap: 5px; flex-direction: column;">
            { props.children.clone() }
        </div>
    }
}
