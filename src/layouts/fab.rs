use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FabProps {
    pub children: Html,
}

#[function_component(Fab)]
pub fn fab(props: &FabProps) -> Html {
    html! {
        <div style=r#"
            position: fixed; bottom: env(safe-area-inset-bottom); right: 0em; z-index: 100; /* fab position */
            padding: 1em; padding-bottom: calc(env(safe-area-inset-bottom, 16px) + 1em); /* edge gap */
            display: flex; gap: 5px; flex-direction: column; /* children gap */
        "#>
            { props.children.clone() }
        </div>
    }
}
