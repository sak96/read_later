use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FabProps {
    pub children: Html,
}

#[component(Fab)]
pub fn fab(props: &FabProps) -> Html {
    html! {
        <div style=r#"
            position: fixed;
            bottom: var(--safe-area-inset-bottom, 0);
            right: 0em;
            z-index: 100;
            padding: 1em;
            display: flex;
            flex-direction: column;
            gap: 5px;
        "#>
            { props.children.clone() }
        </div>
    }
}
