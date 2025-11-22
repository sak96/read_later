use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FabProps {
    pub show_menu: bool,
    pub on_toggle: Callback<MouseEvent>,
    pub on_add: Callback<MouseEvent>,
    pub on_settings: Callback<MouseEvent>,
}

#[function_component(Fab)]
pub fn fab(props: &FabProps) -> Html {
    html! {
        <div class="fab">
            if props.show_menu {
                <div class="fab-menu">
                    <button onclick={props.on_settings.clone()}>
                        <i class="ti ti-settings"></i>
                    </button>
                    <button onclick={props.on_add.clone()}>
                        <i class="ti ti-plus"></i>
                    </button>
                </div>
            }
            <button onclick={props.on_toggle.clone()}>
                <i class="ti ti-menu-2"></i>
            </button>
        </div>
    }
}
