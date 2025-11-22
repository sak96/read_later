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
        <div style="position: sticky; bottom: 0em; padding: 1em;">
            if props.show_menu {
                <button onclick={props.on_add.clone()}>
                    <i class="ti ti-plus"></i>
                </button>
            }
            <div>
                <button onclick={props.on_toggle.clone()}>
                    <i class="ti ti-menu-2"></i>
                </button>
                if props.show_menu {
                    <button onclick={props.on_settings.clone()}>
                        <i class="ti ti-settings-filled"></i>
                    </button>
                }
            </div>
        </div>
    }
}
